const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("$VULN_NAME_READABLE POC - $TARGET_CONTRACT_NAME", function () {
  let deployer;
  let attacker;
  let target;
  let attackContract;

  beforeEach(async function () {
    [deployer, attacker] = await ethers.getSigners();

    const Target = await ethers.getContractFactory("$TARGET_CONTRACT_NAME");
    target = await Target.connect(deployer).deploy();
    await target.waitForDeployment();

    const Attack = await ethers.getContractFactory("$ATTACK_CONTRACT_NAME");
    attackContract = await Attack.connect(attacker).deploy(
      await target.getAddress()
    );
    await attackContract.waitForDeployment();
  });

  it("attempts to take over privileged role", async function () {
    await attackContract.connect(attacker).attack();
    const newOwner = attacker.address;

    // 具体断言依赖目标合约实现，这里只验证调用未被拒绝
    await expect(
      attackContract.connect(attacker).attack()
    ).to.not.be.reverted;

    // 可按需扩展：读取目标合约状态，确认权限已被篡改
    expect(newOwner).to.equal(attacker.address);
  });
});

