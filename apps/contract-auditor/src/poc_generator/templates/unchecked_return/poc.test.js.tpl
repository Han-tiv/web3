const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("$VULN_NAME_READABLE POC - $TARGET_CONTRACT_NAME", function () {
  let deployer;
  let target;
  let attackContract;

  beforeEach(async function () {
    [deployer] = await ethers.getSigners();

    const Target = await ethers.getContractFactory("$TARGET_CONTRACT_NAME");
    target = await Target.connect(deployer).deploy();
    await target.waitForDeployment();

    const Attack = await ethers.getContractFactory("$ATTACK_CONTRACT_NAME");
    attackContract = await Attack.connect(deployer).deploy(
      await target.getAddress()
    );
    await attackContract.waitForDeployment();
  });

  it("forwards call without checking return value", async function () {
    await expect(
      attackContract.attack(ethers.ZeroAddress, 0)
    ).to.not.be.reverted;
  });
});

