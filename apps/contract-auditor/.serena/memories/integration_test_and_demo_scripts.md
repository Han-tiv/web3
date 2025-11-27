为 AI 智能合约审计系统增加了一套端到端测试与演示脚本：

1. test_system.sh
   - 位置：项目根目录
   - 功能：一键执行系统级功能验证，包括：
     - Solidity 解析器对 examples/VulnerableVault.sol 的解析
     - VulnerabilityDetector 静态规则检测（tx.origin 规则）
     - POCGenerator 生成重入漏洞 POC（使用 data/test_poc 作为输出目录）
     - CLI 审计入口 src/main.py 对 examples/VulnerableContract.sol 的审计（--no-poc，只验证流程与报告生成）
     - VulnerabilityRuleEngine 规则加载与配置检查
   - 特点：
     - 自动激活 .venv 虚拟环境（如存在）
     - 输出分步骤中文说明与 emoji 结果标记

2. tests/test_integration.py
   - 位置：tests/
   - 使用 pytest，覆盖核心模块：
     - SolidityParser：解析 VulnerableVault.sol，验证解析出的合约与函数数量
     - VulnerabilityDetector：检测 tx.origin 与 delegatecall 两类典型风险
     - POCGenerator：生成重入漏洞 POC，断言目录、测试脚本、Hardhat 配置、README 均存在
     - VulnerabilityRuleEngine：验证内置规则加载、match_rules 能匹配 tx.origin 规则，以及 analyze_fund_impact 对资金影响等级的判定（"直接盗币" 和 "价值减少"）

3. pytest.ini
   - 配置 pytest 搜索路径和命名约定：tests/ 目录、test_* 文件、Test* 类、test_* 函数，并启用 -v --tb=short

4. demo.py
   - 位置：项目根目录
   - 功能：交互式演示完整审计流程：
     - 从 config.yaml 初始化 ContractAuditor
     - 在 examples/VulnerableVault.sol 和 examples/VulnerableContract.sol 中选择存在的示例合约
     - 调用 auditor.audit(contract_path, output_dir="data/demo_results", generate_poc=True, use_rules=True)
     - 打印审计结果摘要（合约数、漏洞总数、按严重级别统计）
     - 展示前 3 条漏洞详情及其 POC 路径（如存在）
     - 提示报告 JSON 输出位置 data/demo_results/audit_report_*.json
   - 说明：避免强依赖 LLM 配置，文案中提示可通过禁用规则或 LLM 加速测试。

使用建议：
- 快速检查环境与核心功能：运行 `./test_system.sh`
- 在本地开发或 CI 集成测试中：运行 `pytest`（需提前安装 pytest 依赖）
- 对外演示系统能力：运行 `./demo.py` 展示从配置加载到报告生成的完整链路。
