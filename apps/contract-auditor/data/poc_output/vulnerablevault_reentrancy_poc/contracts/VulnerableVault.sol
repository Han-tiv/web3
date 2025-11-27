// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title VulnerableVault
 * @notice 示例合约 - 包含多种典型漏洞，用于测试审计系统
 */
contract VulnerableVault {
    address public owner;
    mapping(address => uint256) public balances;
    uint256 public totalSupply;

    event Deposit(address indexed user, uint256 amount);
    event Withdrawal(address indexed user, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    // 漏洞1: 权限绕过 - approve 函数缺少权限检查
    function approve(address spender, uint256 amount) public {
        // BUG: 任何人都可以批准他人花费
        balances[spender] += amount;
    }

    // 漏洞2: 精度丢失 - downscale/upscale 不匹配
    function deposit(uint256 amount) public payable {
        // BUG: 精度处理不当
        uint256 scaled = amount / 1e18;  // downscale
        balances[msg.sender] += scaled * 1e17;  // upscale 错误（应该是 1e18）
        totalSupply += scaled * 1e17;
        emit Deposit(msg.sender, amount);
    }

    // 漏洞3: 重入攻击
    function withdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount, "Insufficient balance");

        // BUG: 在状态更新前进行外部调用
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");

        balances[msg.sender] -= amount;  // 状态更新太晚
        emit Withdrawal(msg.sender, amount);
    }

    // 漏洞4: Admin 滥用 - 无时间锁的紧急提款
    function emergencyWithdraw() public onlyOwner {
        // BUG: 管理员可以随时提取所有资金，无延迟
        payable(owner).transfer(address(this).balance);
    }

    // 漏洞5: 整数溢出（Solidity < 0.8.0）
    function unsafeAdd(uint256 a, uint256 b) public pure returns (uint256) {
        // 在旧版本中会溢出
        return a + b;
    }

    // 漏洞6: 跨合约交互 - 未验证返回值
    function externalCall(address target, bytes memory data) public onlyOwner returns (bool) {
        // BUG: 未检查调用是否成功
        target.call(data);
        return true;  // 总是返回 true
    }

    // 正确的函数示例（用于对比）
    function safeWithdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount, "Insufficient balance");

        // 正确: 先更新状态
        balances[msg.sender] -= amount;

        // 再进行外部调用
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");

        emit Withdrawal(msg.sender, amount);
    }

    receive() external payable {}
}
