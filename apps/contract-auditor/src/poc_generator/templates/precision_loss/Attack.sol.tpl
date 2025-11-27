// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的精度丢失示例，仅用于安全研究
interface IPrecisionTarget {
    function $PRECISION_FUNCTION(uint256 amount) external returns (uint256);
}

contract $ATTACK_CONTRACT_NAME {
    IPrecisionTarget public target;
    uint256 public lastResult;

    constructor(address _target) {
        target = IPrecisionTarget(_target);
    }

    function trigger(uint256 amount) external {
        lastResult = target.$PRECISION_FUNCTION(amount);
    }
}
