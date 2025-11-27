# 🚀 快速开始指南

## 📦 安装

### 1. 安装依赖

```bash
cd apps/contract-auditor
pip install -r requirements.txt
```

### 2. 安装 Ollama（推荐本地运行）

```bash
# 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 拉取推理模型
ollama pull deepseek-coder-v2:32b

# 拉取编码模型
ollama pull qwen2.5:14b

# 启动 Ollama 服务
ollama serve
```

### 3. 配置

编辑 `config.yaml`，确认 LLM 配置正确：

```yaml
llm:
  reasoning_model: "deepseek-coder-v2:32b"
  coding_model: "qwen2.5:14b"
  api_url: "http://localhost:11434/api/generate"
  max_rounds: 27
  min_confidence: 0.87
```

## 🎯 运行审计

### 命令行模式

```bash
# 审计示例合约
python -m src.main examples/VulnerableVault.sol

# 指定输出目录
python -m src.main examples/VulnerableVault.sol data/my_results
```

### 输出示例

```
🚀 初始化 AI Contract Auditor
🧠 推理模型: deepseek-coder-v2:32b
💻 编码模型: qwen2.5:14b
✅ 初始化完成

📄 开始审计: examples/VulnerableVault.sol

================================================================================
📊 Step 1/4: 解析合约
================================================================================
📄 解析合约: VulnerableVault.sol
  📦 合约: VulnerableVault
     函数: 8 个
     状态变量: 3 个
✅ 解析完成，共 1 个合约
💾 已保存 JSON: data/results/parsed_contracts.json

================================================================================
🔍 Step 2/4: 多轮 ReAct 推理（25-27轮）
================================================================================
🔄 Round 1/27 - 推理中...
   ✅ 发现漏洞: ACCESS_CONTROL (置信度 0.85)
🔄 Round 2/27 - 推理中...
   ✅ 发现漏洞: PRECISION_LOSS (置信度 0.88)
🔄 Round 3/27 - 推理中...
   ✅ 发现漏洞: REENTRANCY (置信度 0.92)
...
✅ 分析完成！共 27 轮，发现 6 个漏洞

================================================================================
💻 Step 3/4: 生成 POC 脚本
================================================================================
⏭️  POC 生成暂时跳过（待实现）

================================================================================
📝 Step 4/4: 生成审计报告
================================================================================

✅ 审计完成！
📊 发现漏洞: 6 个
⏱️  耗时: 1234.5 秒
🔢 Tokens: 156789
📁 报告: data/results/audit_report_20251116_142615.json

================================================================================
📊 漏洞摘要
================================================================================

🔴 HIGH - REENTRANCY
   置信度: 92.0%
   可利用性: HIGH
   描述: withdraw 函数在状态更新前进行外部调用
   位置: VulnerableVault.withdraw

🟡 MEDIUM - PRECISION_LOSS
   置信度: 88.0%
   可利用性: MEDIUM
   描述: deposit 函数 downscale/upscale 缩放因子不匹配
   位置: VulnerableVault.deposit

🟡 MEDIUM - ACCESS_CONTROL
   置信度: 85.0%
   可利用性: HIGH
   描述: approve 函数缺少权限检查
   位置: VulnerableVault.approve
```

## 📊 查看报告

审计报告保存为 JSON 格式，包含：

```json
{
  "metadata": {
    "auditor": "AI Contract Auditor",
    "contract_path": "examples/VulnerableVault.sol",
    "audit_date": "2025-11-16T14:26:15",
    "duration_seconds": 1234.5,
    "tokens_used": 156789
  },
  "summary": {
    "total_contracts": 1,
    "total_vulnerabilities": 6,
    "high_risk": 2,
    "medium_risk": 3,
    "low_risk": 1,
    "confidence": 0.89
  },
  "vulnerabilities": [
    {
      "type": "REENTRANCY",
      "severity": "HIGH",
      "confidence": 0.92,
      "exploitability": "HIGH",
      "description": "withdraw 函数在状态更新前进行外部调用",
      "location": "VulnerableVault.withdraw",
      "recommendation": "使用 Checks-Effects-Interactions 模式"
    }
  ]
}
```

## 🎨 Web 界面（开发中）

```bash
# 启动 Web 界面
streamlit run ui/app.py
```

访问 `http://localhost:8501` 查看交互式审计面板。

## ⚙️ 高级配置

### 调整推理轮次

```yaml
llm:
  max_rounds: 20  # 减少轮次，加快速度
  min_confidence: 0.85  # 降低阈值，发现更多潜在漏洞
```

### 使用云端 API

```yaml
llm:
  reasoning_model: "deepseek-coder"
  api_url: "https://api.deepseek.com/v1/chat/completions"
  api_key: "your-api-key-here"
```

### 关注特定漏洞类型

```yaml
audit:
  focus_areas:
    - "重入攻击"
    - "权限绕过"
    # 注释掉不关心的类型
    # - "Gas 优化"
```

## 🧪 测试

```bash
# 运行测试
pytest tests/

# 测试特定模块
pytest tests/test_parser.py -v
```

## 📚 下一步

1. ✅ 审计你的第一个合约
2. 📖 阅读 [设计文档](docs/DESIGN.md)
3. 🔧 自定义 [Prompt 模板](prompts/)
4. 🤝 贡献代码或反馈问题

## ❓ 常见问题

### Q: Ollama 启动失败？
A: 确保端口 11434 未被占用，或修改 `config.yaml` 中的 `api_url`

### Q: 推理太慢？
A: 可以减少 `max_rounds` 或使用更小的模型（如 `qwen2.5:7b`）

### Q: 误报太多？
A: 提高 `min_confidence` 阈值（如 0.90）

### Q: 想要更详细的日志？
A: 修改 `config.yaml` 中的 `logging.level: DEBUG`

## 📞 获取帮助

- 📖 查看完整文档：[docs/](docs/)
- 💬 提交 Issue：[GitHub Issues](../../issues)
- 🐦 关注羊博士：[@ybspro_official](https://x.com/ybspro_official)
