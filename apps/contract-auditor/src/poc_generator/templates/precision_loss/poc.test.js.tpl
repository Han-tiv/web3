const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("$VULN_NAME_READABLE POC - $TARGET_CONTRACT_NAME", function () {
  let deployer;
  let target;
  let helper;

  beforeEach(async function () {
    [deployer] = await ethers.getSigners();

    const Target = await ethers.getContractFactory("$TARGET_CONTRACT_NAME");
    target = await Target.connect(deployer).deploy();
    await target.waitForDeployment();

    const Helper = await ethers.getContractFactory("$ATTACK_CONTRACT_NAME");
    helper = await Helper.connect(deployer).deploy(await target.getAddress());
    await helper.waitForDeployment();
  });

  it("demonstrates precision loss on small amounts", async function () {
    const amount = 1n;
    await expect(helper.trigger(amount)).to.not.be.reverted;

    const result = await helper.lastResult();
    expect(result).to.be.a("bigint");
  });
});
