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

  it("replays a crafted signature payload", async function () {
    const signer = deployer;
    const value = 0n;
    const message = ethers.solidityPackedKeccak256(
      ["address", "uint256"],
      [await target.getAddress(), value]
    );
    const signature = await signer.signMessage(
      ethers.getBytes(message)
    );

    await expect(
      attackContract.replay(signer.address, value, signature)
    ).to.not.be.reverted;
  });
});

