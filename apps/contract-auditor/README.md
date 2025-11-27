# 🔍 AI智能合约审计系统

基于 ReAct 框架的多轮推理审计 Agent，集成静态规则检测、POC 自动生成与 Web 监控界面。

> 设计灵感与理论基础来自 **羊博士(@ybspro_official)** 的 AI 智能合约审计方案。

## ✨ 核心特性

- **🤖 ReAct 多轮推理**：25-27 轮 Reason → Act → Observe 迭代分析
- **🔎 双引擎检测**：AI 推理 + 静态规则混合检测
- **⚡ 完整 POC 生成**：7 种漏洞类型的可执行 Hardhat 测试脚本
- **🌐 Web 实时监控**：Streamlit 界面实时展示审计进度
- **📊 资金影响分析**：CRITICAL/HIGH/MEDIUM/LOW/NONE 五档评估
- **🎯 高置信度**：87%+ 阈值过滤，降低误报率

## 🏗️ 架构设计

```
┌─────────────────┐
│   Solidity      │
│   Contract      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│         SolidityParser                  │
│  (AST解析:合约/函数/状态变量/继承)      │
└────────┬─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│      Hybrid Detection Engine            │
│                                         │
│  ┌──────────────┐  ┌──────────────────┐ │
│  │ Static Rules │  │   ReAct Engine   │ │
│  │   Detector   │  │   (25-27 rounds) │ │
│  │  (12 rules)  │  │  LLM推理+搜索     │ │
│  └──────────────┘  └──────────────────┘ │
└────────┬─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│     Vulnerability Aggregator            │
│   (去重合并+置信度过滤+资金影响评估)     │
└────────┬─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│       POC Generator                     │
│  (生成Hardhat测试+攻击合约+配置文件)     │
└────────┬────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│         Report Builder                  │
│    (JSON/Markdown/PDF多格式报告)        │
└─────────────────────────────────────────┘
```

## 📦 项目结构

> 仅保留主要模块，详细使用示例见 `QUICKSTART.md` 与 `PROJECT_SUMMARY.md`。

```
contract-auditor/
├── src/
│   ├── main.py                    # 主程序入口 (CLI + Web 模式)
│   ├── utils/
│   │   ├── config.py              # 配置管理
│   │   └── llm_client.py          # LLM API 客户端 (DualModelSystem)
│   ├── parser/
│   │   └── solidity_parser.py     # Solidity 解析器
│   ├── analyzer/
│   │   ├── react_engine.py        # ReAct 推理引擎
│   │   └── vulnerability_detector.py  # 静态规则检测器 + 规则引擎适配
│   ├── rules/
│   │   ├── vulnerability_rules.py # 规则引擎 (12 类漏洞 + 资金影响分析)
│   │   └── rules.yaml             # 规则配置 (启用/优先级/权重)
│   ├── poc_generator/
│   │   ├── poc_generator.py       # POC 生成器 (Hardhat + ethers.js)
│   │   ├── models.py              # POC 数据模型定义
│   │   └── templates/             # POC 模板 (7 类漏洞)
│   ├── reporter/
│   │   └── __init__.py            # 报告配置（JSON/Markdown/PDF）
│   ├── web_ui/
│   │   ├── app.py                 # Streamlit Web 主应用
│   │   ├── state.py               # 审计任务状态管理
│   │   ├── tasks.py               # 异步任务封装 (调用 ContractAuditor)
│   │   └── components.py          # UI 组件 (进度、列表、详情、导出)
│   └── web/                       # 旧版多页 Streamlit Web (保留兼容)
│       ├── app.py
│       ├── session_state.py
│       ├── sidebar.py
│       ├── audit_progress.py
│       └── vulnerability_list.py
├── examples/
│   ├── VulnerableVault.sol        # 示例合约 (6 种漏洞)
│   └── VulnerableContract.sol     # 示例合约 (5 种高危漏洞)
├── tests/
│   └── test_integration.py        # 集成测试 (解析/规则/POC/规则引擎)
├── data/                          # 审计结果与测试输出
├── config.yaml                    # 主配置文件
├── audit.sh                       # CLI 启动脚本 (推荐)
├── run_web.sh                     # Web 启动脚本
├── test_system.sh                 # 一键系统测试脚本
├── demo.py                        # 交互式演示脚本
├── QUICKSTART.md                  # 快速开始文档
├── PROJECT_SUMMARY.md             # 项目设计与状态总览
└── README.md                      # 当前主文档
```

## 🚀 快速开始

### 安装依赖

```bash
# 克隆项目
git clone <repo_url>
cd contract-auditor

# 安装 Python 依赖
pip install -r requirements.txt

# 可选：创建虚拟环境
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
```

### 安装 Ollama（本地 LLM，推荐）

```bash
# 参考官网安装 Ollama
# macOS/Linux: https://ollama.ai/download

# 拉取推理模型
ollama pull deepseek-coder-v2:32b

# 拉取编码模型
ollama pull qwen2.5:14b
```

### 配置 LLM 与审计参数

编辑项目根目录下的 `config.yaml`：

```yaml
llm:
  reasoning_model: "deepseek-coder-v2:32b"
  coding_model: "qwen2.5:14b"
  api_url: "http://localhost:11434/api/generate"
  max_rounds: 27
  min_confidence: 0.87
  temperature: 0.7
  max_tokens: 4096

audit:
  focus_areas:
    - "重入攻击"
    - "权限绕过"
    - "精度丢失"
  enable_poc_generation: true
  enable_rule_detection: true
```

## 🧪 使用方式

### CLI 模式 - 审计单个合约

基于 `src/main.py` 中的 `ContractAuditor` 实现。

```bash
# 推荐：通过脚本启动（已封装常用参数）
./audit.sh examples/VulnerableVault.sol

# 或直接调用主程序
python src/main.py examples/VulnerableVault.sol

# 仅规则检测（快速验证）
python src/main.py examples/VulnerableVault.sol --no-poc

# 自定义输出目录
python src/main.py examples/VulnerableVault.sol -o data/my_audit

# 禁用静态规则（仅 AI 推理）
python src/main.py examples/VulnerableVault.sol --no-rules
```

### Web 模式 - 实时监控界面

基于 `src/web_ui/` 的 Streamlit Web 控制台。

```bash
# 启动 Web 界面（脚本方式）
./run_web.sh

# 或直接调用主程序
python src/main.py --web

# 浏览器访问
http://localhost:8501
```

主要能力：

- 文件上传：拖拽上传 `.sol` 文件或选择本地合约
- 审计配置：模型选择、轮次设置、置信度阈值
- 实时进度：展示当前第 X 轮 ReAct 推理的 Reason/Act/Observe 输出
- 统计面板：高/中/低/信息级漏洞数量统计
- 漏洞列表：支持按严重性、置信度、轮次筛选和排序
- 详情视图：
  - 代码片段高亮（前后若干行）
  - 置信度/轮次变化趋势
  - POC 一键下载（ZIP 包）
  - 修复建议与参考链接
- 报告导出：JSON / Markdown / PDF 多格式下载

### 演示模式 - 快速体验

`demo.py` 提供交互式端到端演示。

```bash
./demo.py
```

功能：

- 从 `config.yaml` 初始化 `ContractAuditor`
- 选择 `examples/` 下示例合约进行审计
- 在 `data/demo_results` 下输出审计报告 JSON
- 在终端打印漏洞统计与若干条典型漏洞详情

### 一键测试 - 系统级验证

`test_system.sh` 覆盖解析、规则、POC、CLI 审计与规则配置加载。

```bash
./test_system.sh
```

输出示例（节选）：

```text
🧪 AI智能合约审计系统 - 功能测试

📝 测试1: Solidity解析器
   ✅ 解析成功: 发现 1 个合约

🔎 测试2: 静态规则检测
   ✅ 检测成功: 发现 1 个潜在漏洞

⚡ 测试3: POC生成器
   ✅ POC生成成功: data/test_poc/...

🔍 测试4: CLI审计流程 (仅规则检测)
   ✅ 审计完成,报告已生成

⚙️ 测试5: 规则配置系统
   ✅ 规则引擎加载成功
   ✅ 内置规则数: ...
```

### 单元测试

```bash
pytest
```

覆盖范围：

- SolidityParser 解析测试
- VulnerabilityDetector 静态规则检测
- POCGenerator 生成测试
- VulnerabilityRuleEngine 规则加载与资金影响分析

## 🔍 漏洞检测能力

### 支持的漏洞类型（12 类）

由 `VulnerabilityCategory` 与内置规则定义：

| 类别 | 名称 | 资金影响 | 检测方式 |
|------|------|----------|----------|
| 1 | 重入攻击 | CRITICAL | AI + 规则 |
| 2 | 权限绕过 | HIGH | AI + 规则 |
| 3 | 精度丢失 | MEDIUM | AI + 规则 |
| 4 | 整数溢出 | HIGH | AI + 规则 |
| 5 | 未检查返回值 | MEDIUM | AI + 规则 |
| 6 | 时间戳依赖 | MEDIUM | AI + 规则 |
| 7 | tx.origin 认证 | HIGH | 规则 |
| 8 | delegatecall 漏洞 | CRITICAL | 规则 |
| 9 | selfdestruct 滥用 | CRITICAL | 规则 |
| 10 | 未保护初始化 | HIGH | 规则 |
| 11 | 抢跑攻击 | MEDIUM | 规则 |
| 12 | 签名重放 | HIGH | AI + 规则 |

### POC 生成支持（7 类）

由 `VulnerabilityType` 枚举与模板目录决定：

1. **重入攻击** - `ReentrancyAttack` 合约
2. **权限绕过** - `AccessControlBypass` 合约
3. **精度丢失** - `PrecisionLossExploit` 合约/脚本
4. **整数溢出** - `IntegerOverflowExploit` 合约/脚本
5. **未检查返回值** - `UncheckedReturnExploit` 合约/脚本
6. **时间戳依赖** - `TimestampDependenceExploit` 合约/脚本
7. **签名重放** - `SignatureReplayAttack` 合约/脚本

每个 POC 工程包含：

- ✅ 完整 Hardhat 测试脚本 (`test/*_poc.test.js`)
- ✅ 攻击合约（如需要，位于 `contracts/Attack*.sol`）
- ✅ `hardhat.config.js`
- ✅ `package.json`
- ✅ `.env.example`
- ✅ `README.md`（执行说明）

## ⚙️ 配置文件

### `config.yaml`

关键字段说明：

- `llm.reasoning_model` / `llm.coding_model`：推理模型与代码生成模型
- `llm.max_rounds`：ReAct 最大轮次（推荐 25-27）
- `llm.min_confidence`：整体置信度阈值（推荐 ≥ 0.87）
- `audit.enable_poc_generation`：是否启用 POC 自动生成
- `audit.enable_rule_detection`：是否启用静态规则检测

### `src/rules/rules.yaml`

示例配置（项目已内置）：

```yaml
rules:
  - category: "tx_origin"
    enabled: true
    severity: "HIGH"
    priority: 1

  - category: "delegatecall"
    enabled: true
    severity: "HIGH"
    priority: 1

  - category: "selfdestruct"
    enabled: true
    severity: "HIGH"
    priority: 1

  - category: "unprotected_initialization"
    enabled: true
    severity: "HIGH"
    priority: 2

  - category: "front_running"
    enabled: true
    severity: "MEDIUM"
    priority: 3

fund_impact_weights:
  CRITICAL: 10.0
  HIGH: 7.0
  MEDIUM: 4.0
  LOW: 2.0
  NONE: 0.0
```

## 📈 性能与实践数据

> 以下为实验环境下的典型参考值，实际效果依赖于模型、Prompt 与合约复杂度。

- **覆盖率**：示例合约中的典型 DeFi 漏洞全部命中
- **置信度**：87.1% - 93.5%（多轮累积后）
-,审计时间：~1 小时（27 轮）
- **Token 消耗**：~180k tokens / 合约（云端 API 场景）
- **误报率**：< 5%（在实验数据上，经人工复核）

### 实际案例（Balancer-like DeFi）

- **总发现**：约 180 个（高危 10 + 中危 13 + 低危 3 + 信息 153）
- **关键发现**：Factory 二次验证缺失等逻辑风险（曾被人工遗漏）
- **策略**：通过资金影响 + 置信度综合排序，优先展示高危高置信度问题

## 🛠️ 开发指南

### 添加自定义规则

在不修改内置规则的前提下，可以动态注册自定义规则：

```python
from src.rules.vulnerability_rules import VulnerabilityRule, VulnerabilityCategory, FundImpact
from src.analyzer.vulnerability_detector import VulnerabilityDetector

custom_rule = VulnerabilityRule(
    category=VulnerabilityCategory.DELEGATECALL,
    name="自定义 delegatecall 检查",
    description="检查特定场景下的 delegatecall 使用",
    fund_impact=FundImpact.HIGH,
    severity="HIGH",
    patterns=["delegatecall", "assembly"],
    exclusions=["whitelist"],
    recommendation="为 delegatecall 目标地址添加白名单验证",
    custom_detector=lambda ctx: "your_pattern" in ctx["code"],
)

detector = VulnerabilityDetector()
detector.rule_engine.add_rule(custom_rule)
```

### 扩展 POC 模板

```bash
# 1. 创建新模板目录
mkdir -p src/poc_generator/templates/my_vuln

# 2. 添加测试脚本模板
touch src/poc_generator/templates/my_vuln/poc.test.js.tpl

# 3. 添加攻击合约模板
touch src/poc_generator/templates/my_vuln/Attack.sol.tpl
```

在 `src/poc_generator/models.py` 中扩展漏洞类型：

```python
from enum import Enum

class VulnerabilityType(str, Enum):
    # ... 已有类型 ...
    MY_VULN = "MY_VULN"
```

并在 `src/poc_generator/poc_generator.py` 中注册可读名称与模板目录。

## 📚 参考资料

- [ReAct 框架论文](https://arxiv.org/abs/2210.03629)
- [Smart Contract Best Practices](https://consensys.github.io/smart-contract-best-practices/)
- [SWC Registry](https://swcregistry.io/)
- [OpenZeppelin 文档](https://docs.openzeppelin.com/)
- 羊博士 Web3 安全系列：[@ybspro_official](https://x.com/ybspro_official)

## 🤝 贡献指南

欢迎提交 Issue 和 PR！

### 开发流程

1. Fork 本仓库
2. 创建 feature 分支：`git checkout -b feature/amazing-feature`
3. 提交修改：`git commit -m 'feat: add amazing feature'`
4. 推送到远程：`git push origin feature/amazing-feature`
5. 发起 Pull Request

### 代码规范

- Python：PEP8 + 类型标注（已配置 `pytest.ini` 等）
- Solidity：建议使用 `solhint`/`slither` 做静态检查
- Git 提交：推荐遵循 Conventional Commits

## 📄 许可证

本项目采用 MIT License，详见 `LICENSE` 文件。

## 📞 项目信息

- 项目地址：<repo_url>
- Issue 反馈：<repo_url>/issues
- 文档中心：<docs_url>（可指向团队内部文档或 Wiki）

---

⚡ 文档由 AI 助手自动生成，并会随着实现演进持续更新。
