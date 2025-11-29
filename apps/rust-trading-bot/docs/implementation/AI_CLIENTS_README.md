# AI客户端使用指南

本项目现已支持三个AI服务提供商，所有客户端接口完全一致，可无缝切换。

## 支持的AI提供商

### 1. DeepSeek (默认，已集成)
- **文件**: `src/deepseek_client.rs`
- **模型**: `deepseek-chat`
- **API端点**: `https://api.deepseek.com/v1`
- **环境变量**: `DEEPSEEK_API_KEY`

### 2. Grok (X.AI) ✨ NEW
- **文件**: `src/grok_client.rs`
- **模型**: `grok-2-1212`
- **API端点**: `https://api.x.ai/v1`
- **环境变量**: `GROK_API_KEY`

### 3. Gemini (Google) ✨ NEW
- **文件**: `src/gemini_client.rs`
- **模型**: `gemini-2.0-flash-exp`
- **API端点**: `https://generativelanguage.googleapis.com/v1beta`
- **环境变量**: `GEMINI_API_KEY`

## 统一接口

所有三个客户端提供完全相同的接口：

```rust
// 核心方法
pub async fn analyze_market(&self, prompt: &str) -> Result<TradingSignal>;
pub async fn analyze_position_management(&self, prompt: &str) -> Result<PositionManagementDecision>;

// Prompt构建方法
pub fn build_prompt(&self, klines: &[Kline], indicators: &TechnicalIndicators, current_price: f64, position: Option<&Position>) -> String;
pub fn build_entry_analysis_prompt(&self, symbol: &str, alert_type: &str, ...) -> String;
pub fn build_position_management_prompt(&self, symbol: &str, side: &str, ...) -> String;
```

## 快速开始

### 1. 添加到代码中

```rust
use rust_trading_bot::deepseek_client::DeepSeekClient;
use rust_trading_bot::grok_client::GrokClient;
use rust_trading_bot::gemini_client::GeminiClient;

// 选择一个客户端
let deepseek = DeepSeekClient::new(std::env::var("DEEPSEEK_API_KEY")?);
let grok = GrokClient::new(std::env::var("GROK_API_KEY")?);
let gemini = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

// 使用相同的接口
let signal = deepseek.analyze_market(&prompt).await?;
// 或
let signal = grok.analyze_market(&prompt).await?;
// 或
let signal = gemini.analyze_market(&prompt).await?;
```

### 2. 配置环境变量

在`.env`文件中添加对应的API密钥：

```bash
# 选择你想使用的AI提供商
DEEPSEEK_API_KEY=sk-xxx
GROK_API_KEY=xai-xxx
GEMINI_API_KEY=AIzaSxxx
```

### 3. 在主程序中切换

如果要在`integrated_ai_trader`中切换AI提供商，修改初始化代码：

```rust
// 当前使用DeepSeek
let deepseek_client = Arc::new(DeepSeekClient::new(deepseek_api_key));

// 切换到Grok
let grok_client = Arc::new(GrokClient::new(grok_api_key));

// 切换到Gemini
let gemini_client = Arc::new(GeminiClient::new(gemini_api_key));
```

## 数据结构复用

所有客户端共享以下数据结构（定义在`deepseek_client.rs`）：

```rust
pub struct TradingSignal {
    pub signal: String,           // "BUY", "SELL", "SKIP"
    pub confidence: String,        // "HIGH", "MEDIUM", "LOW"
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub reason: String,
}

pub struct PositionManagementDecision {
    pub action: String,                    // "HOLD", "PARTIAL_CLOSE", "FULL_CLOSE"
    pub close_percentage: Option<f64>,
    pub limit_price: Option<f64>,
    pub reason: String,
    pub profit_potential: String,
    pub optimal_exit_price: Option<f64>,
    pub confidence: String,
}

pub struct Kline { ... }
pub struct TechnicalIndicators { ... }
pub struct Position { ... }
```

## API格式差异（已内部处理）

尽管三个提供商的API格式不同，但已在客户端内部统一处理：

| 提供商 | 请求格式 | 响应格式 | Token计数 |
|--------|----------|----------|-----------|
| DeepSeek | OpenAI兼容 | `choices[].message.content` | `usage.total_tokens` |
| Grok | OpenAI兼容 | `choices[].message.content` | `usage.total_tokens` |
| Gemini | `contents/parts` | `candidates[].content.parts[].text` | `usageMetadata` |

## 性能建议

1. **DeepSeek**: 成本低，速度快，适合高频交易
2. **Grok**: X.AI最新模型，推理能力强
3. **Gemini**: Google的最新模型，多模态能力强

## 注意事项

⚠️ **当前状态**: 新增的Grok和Gemini客户端**未集成**到主交易程序中，仅提供独立模块供测试和评估使用。

如需集成到主程序，需要：
1. 修改`integrated_ai_trader.rs`中的客户端初始化
2. 添加环境变量配置选择器（如`AI_PROVIDER=deepseek|grok|gemini`）
3. 进行充分的回测验证

## 示例代码

完整示例见各客户端的使用方法，接口签名完全一致。

---

**创建日期**: 2025-11-10
**版本**: v1.0
**维护者**: Trading Bot Team
