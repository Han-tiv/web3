// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的时间戳依赖示例，仅用于安全研究
interface ITimestampTarget {
    function $TIMELOCK_FUNCTION() external;
}

contract $ATTACK_CONTRACT_NAME {
    ITimestampTarget public target;

    constructor(address _target) {
        target = ITimestampTarget(_target);
    }

    function attack() external {
        target.$TIMELOCK_FUNCTION();
    }
}

