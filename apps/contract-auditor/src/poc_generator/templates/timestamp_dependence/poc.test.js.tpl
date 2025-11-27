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

  it("calls time-sensitive function around current block timestamp", async function () {
    const block = await ethers.provider.getBlock("latest");
    expect(block.timestamp).to.be.a("number");

    await expect(attackContract.attack()).to.not.be.reverted;
  });
});

