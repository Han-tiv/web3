// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的未检查返回值示例，仅用于安全研究
interface IUncheckedTarget {
    function $UNSAFE_CALL(address to, uint256 amount) external returns (bool);
}

contract $ATTACK_CONTRACT_NAME {
    IUncheckedTarget public target;

    constructor(address _target) {
        target = IUncheckedTarget(_target);
    }

    function attack(address to, uint256 amount) external {
        bool ok = target.$UNSAFE_CALL(to, amount);
        // 未对 ok 做任何处理，模拟上层合约忽略返回值导致风险
        ok;
    }
}

