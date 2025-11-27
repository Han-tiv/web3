// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VulnerableContract {
    address public owner;
    mapping(address => uint256) public balances;
    bool public initialized;

    // 漏洞1: 未保护初始化
    function initialize(address _owner) public {
        owner = _owner; // 缺少 initialized 检查
    }

    // 漏洞2: tx.origin 认证
    function withdrawAll() public {
        require(tx.origin == owner, "Not owner"); // 应使用 msg.sender
        payable(msg.sender).transfer(address(this).balance);
    }

    // 漏洞3: 不受控 delegatecall
    function execute(address target, bytes memory data) public {
        (bool success, ) = target.delegatecall(data); // target 可被用户控制
        require(success);
    }

    // 漏洞4: 无保护自毁
    function destroy() public {
        selfdestruct(payable(msg.sender)); // 缺少 onlyOwner
    }

    // 漏洞5: 抢跑攻击
    function swap(uint256 amount) public {
        // 缺少 deadline 和滑点保护
        uint256 price = getCurrentPrice();
        uint256 output = amount * price;
        balances[msg.sender] += output;
    }

    function getCurrentPrice() public view returns (uint256) {
        return block.timestamp % 100; // 时间戳依赖
    }
}

