# $PROJECT_NAME

> 自动生成的 POC 工程，用于复现 **$VULN_NAME_READABLE**（$VULN_TYPE）漏洞。

⚠️ **安全警告**

- 本工程仅用于本地或测试网环境的安全研究与教学。
- 严禁在未经授权的生产环境或主网上使用本工程中的脚本和攻击合约。
- 由此产生的任何法律责任由使用者自行承担。

## 目录结构

- `contracts/`：目标合约与自动生成的攻击合约
- `test/`：基于 Hardhat + ethers.js 的漏洞复现测试
- `hardhat.config.js`：Hardhat 配置（支持本地与自定义网络）
- `package.json`：npm 依赖与脚本
- `.env`：RPC 与私钥配置（根据 `.env.example` 复制修改）

## 快速开始

```bash
# 1. 安装依赖
npm install

# 2. 配置环境变量
cp .env.example .env
# 根据需要修改 RPC_URL / PRIVATE_KEY / TARGET_CONTRACT_ADDRESS 等字段

# 3. 运行 POC 测试
npx hardhat test
```

## 漏洞信息

- 类型：$VULN_NAME_READABLE（$VULN_TYPE）
- 目标合约：$TARGET_CONTRACT_NAME
- 目标函数：$TARGET_FUNCTION_NAME
- 描述：$VULN_DESCRIPTION

## 运行说明

测试脚本位于 `test/` 目录，默认使用 Hardhat 内置网络。
如需连接实际链（测试网/主网 fork），请在 `.env` 中配置 `RPC_URL` 和 `PRIVATE_KEY`，
并在 `hardhat.config.js` 中使用 `custom` 网络。

