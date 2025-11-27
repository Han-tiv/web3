//! Gemini ETH-USDT 独立分析器
//! 专门用于分析ETH-USDT合约交易策略

use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use regex::Regex;
use rust_trading_bot::binance_client::{BinanceClient, OpenOrder};
use rust_trading_bot::database::{Database, PendingTpSlRecord, PendingTpSlStatus};
use rust_trading_bot::exchange_trait::{ExchangeClient, Position};
use rust_trading_bot::gemini_client::GeminiClient;
use rust_trading_bot::market_data_fetcher::Kline;
use serde_json::json;
use std::env;
use tokio;

const SYMBOL: &str = "ETHUSDT";
const ANALYSIS_INTERVAL_SECONDS: u64 = 390; // 6分30秒分析一次
const LEVERAGE: u32 = 20;
const PENDING_TPSL_MAX_RETRY: usize = 3;
const MIN_DYNAMIC_CAPITAL: f64 = 0.5;
const MAX_DYNAMIC_CAPITAL: f64 = 0.5;
const TRIGGER_ORDER_MAX_WAIT_ATTEMPTS: u32 = 6; // 最多轮询 6 次 (约 30 秒)
const TRIGGER_ORDER_POLL_INTERVAL_SECS: u64 = 5;
const STOP_LOSS_RETRY_OFFSET: f64 = 25.0; // -2021 报错时，距离当前价 25U 重新设置

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 Gemini ETH-USDT 分析器启动");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📊 分析币种: {}", SYMBOL);
    info!("⏰ 分析间隔: {}秒", ANALYSIS_INTERVAL_SECONDS);
    info!("");

    // 初始化客户端
    let binance_api_key = env::var("BINANCE_API_KEY").context("缺少BINANCE_API_KEY环境变量")?;
    let binance_secret = env::var("BINANCE_SECRET").context("缺少BINANCE_SECRET环境变量")?;
    let gemini_api_key = env::var("GEMINI_API_KEY")
        .or_else(|_| env::var("GOOGLE_GEMINI_API_KEY"))
        .context("缺少GEMINI_API_KEY或GOOGLE_GEMINI_API_KEY环境变量")?;

    let binance = BinanceClient::new(binance_api_key, binance_secret, false);
    let gemini = GeminiClient::new(gemini_api_key);
    std::fs::create_dir_all("data").ok();
    let db = Database::new("data/trading.db").context("初始化数据库失败")?;

    info!("✅ Binance客户端已初始化");
    info!("✅ Gemini客户端已初始化");
    info!("✅ 数据库已初始化");
    info!("");

    // 主循环
    loop {
        match analyze_eth_usdt(&binance, &gemini, &db).await {
            Ok(_) => info!("✅ 分析完成\n"),
            Err(e) => error!("❌ 分析失败: {}\n", e),
        }

        info!(
            "⏳ 等待 {} 秒后进行下一次分析...",
            ANALYSIS_INTERVAL_SECONDS
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(ANALYSIS_INTERVAL_SECONDS)).await;
    }
}

async fn analyze_eth_usdt(
    binance: &BinanceClient,
    gemini: &GeminiClient,
    db: &Database,
) -> Result<()> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🔍 开始分析 ETH-USDT");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 1. 获取多周期K线数据
    info!("📈 获取K线数据...");
    let klines_1m = convert_raw_klines(binance.get_klines(SYMBOL, "1m", Some(50)).await?);
    let klines_5m = convert_raw_klines(binance.get_klines(SYMBOL, "5m", Some(50)).await?);
    let klines_15m = convert_raw_klines(binance.get_klines(SYMBOL, "15m", Some(50)).await?);
    let klines_1h = convert_raw_klines(binance.get_klines(SYMBOL, "1h", Some(50)).await?);
    let klines_4h = convert_raw_klines(binance.get_klines(SYMBOL, "4h", Some(50)).await?);
    info!("   ✓ 1分钟: {} 根", klines_1m.len());
    info!("   ✓ 5分钟: {} 根", klines_5m.len());
    info!("   ✓ 15分钟: {} 根", klines_15m.len());
    info!("   ✓ 1小时: {} 根", klines_1h.len());
    info!("   ✓ 4小时: {} 根", klines_4h.len());

    // 2. 计算技术指标
    info!("📊 计算技术指标...");
    let indicators = calculate_indicators(&klines_1h)?;
    let current_price = indicators["current_price"].as_f64().unwrap_or_default();
    info!("   ✓ 当前价格: ${:.2}", current_price);

    // 3. 获取当前持仓
    info!("💼 获取持仓信息...");
    let positions = binance.get_positions().await?;
    let eth_position = positions
        .iter()
        .find(|p| p.symbol == SYMBOL && p.size.abs() > f64::EPSILON);
    let current_position = eth_position.cloned();

    if let Some(pos) = eth_position {
        info!(
            "   ✓ 持仓: {} {}, 入场价: ${:.2}, 未实现盈亏: ${:.2}",
            pos.side, pos.size, pos.entry_price, pos.pnl
        );
    } else {
        info!("   ✓ 无持仓");
    }

    // 4. 获取止盈止损订单
    info!("📋 获取止盈止损订单...");
    let all_orders = binance.get_open_orders(Some(SYMBOL)).await?;
    let tpsl_orders: Vec<_> = all_orders
        .iter()
        .filter(|o| {
            o.order_type == "STOP_MARKET"
                || o.order_type == "TAKE_PROFIT_MARKET"
                || o.order_type == "STOP"
                || o.order_type == "TAKE_PROFIT"
        })
        .collect();
    info!("   ✓ 止盈止损订单: {} 个", tpsl_orders.len());

    if let Some(position) = current_position.as_ref() {
        let normalized_side = position.side.to_ascii_uppercase();
        let (has_tp, has_sl) = has_tpsl_orders_for_position(&tpsl_orders, &normalized_side);
        if !has_tp || !has_sl {
            let need_tp = !has_tp;
            let need_sl = !has_sl;
            let missing_desc = match (need_tp, need_sl) {
                (true, true) => "止盈与止损",
                (true, false) => "止盈",
                (false, true) => "止损",
                _ => "止盈/止损",
            };
            info!(
                "🛡 当前 {} 仓位缺少{}，尝试根据待处理记录自动补设。",
                position.side, missing_desc
            );

            match db
                .next_pending_tpsl(SYMBOL, &normalized_side)
                .context("查询待处理止盈止损记录失败")?
            {
                Some(pending) => {
                    let mut applied = false;
                    let mut last_error: Option<String> = None;
                    let record_id = pending.id.unwrap_or_default();

                    for attempt in 1..=PENDING_TPSL_MAX_RETRY {
                        match fulfill_pending_tpsl_orders(
                            binance, position, &pending, need_tp, need_sl,
                        )
                        .await
                        {
                            Ok(_) => {
                                info!(
                                    "✅ 已在第 {} 次尝试中为 {} 仓位补齐止盈/止损 (记录ID: {}).",
                                    attempt, position.side, record_id
                                );
                                applied = true;
                                break;
                            }
                            Err(err) => {
                                let err_msg = err.to_string();
                                warn!(
                                    "⚠️ 第 {} 次补设止盈/止损失败 (记录ID: {}): {}",
                                    attempt, record_id, err_msg
                                );
                                last_error = Some(err_msg);
                                if attempt < PENDING_TPSL_MAX_RETRY {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500))
                                        .await;
                                }
                            }
                        }
                    }

                    if applied {
                        if let Some(id) = pending.id {
                            db.update_pending_tpsl_status(id, PendingTpSlStatus::Completed)
                                .context("更新待处理止盈止损状态失败")?;
                        } else {
                            warn!("⚠️ pending_tpsl 记录缺少主键，无法标记完成。");
                        }
                    } else {
                        if let Some(id) = pending.id {
                            db.update_pending_tpsl_status(id, PendingTpSlStatus::Failed)
                                .context("更新待处理止盈止损状态失败")?;
                        }
                        if let Some(err_msg) = last_error {
                            warn!(
                                "⚠️ 自动补设止盈止损失败(记录ID: {}): {}",
                                record_id, err_msg
                            );
                        }
                    }
                }
                None => {
                    info!(
                        "ℹ️ 未找到 {} 仓位对应的待处理止盈止损记录，等待下一轮。",
                        position.side
                    );
                }
            }
        }
    }

    // 5. 获取计划委托
    let trigger_refs: Vec<_> = all_orders
        .iter()
        .filter(|o| (o.order_type == "STOP" || o.order_type == "TAKE_PROFIT") && o.status == "NEW")
        .collect();
    info!("   ✓ 计划委托订单: {} 个", trigger_refs.len());

    // 6. 构造详细prompt
    info!("📝 构造分析prompt...");
    let prompt = build_analysis_prompt(
        &klines_1m,
        &klines_5m,
        &klines_15m,
        &klines_1h,
        &klines_4h,
        &indicators,
        eth_position,
        &tpsl_orders,
        &trigger_refs,
    );

    // 7. 调用Gemini分析
    info!("🤖 调用Gemini AI进行分析...");
    let analysis = gemini.analyze(&prompt).await?;

    // 8. 输出分析结果
    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📝 Gemini 分析结果:");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    println!("{}", analysis);
    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let capital = calculate_dynamic_capital(db).await;
    info!("💰 本次动态本金: {:.2} USDT", capital);
    execute_trade_action(binance, &analysis, current_position.clone(), db, capital).await?;

    let actions_payload = parse_structured_actions(&analysis)
        .map(|signal| actions_to_json(&signal.actions))
        .and_then(|value| match serde_json::to_string(&value) {
            Ok(text) => Some(text),
            Err(err) => {
                warn!("⚠️ 无法序列化动作指令，已忽略: {}", err);
                None
            }
        });

    match db.save_analysis_log(
        SYMBOL,
        &analysis,
        current_price,
        &indicators,
        current_position.as_ref(),
        actions_payload.as_deref(),
    ) {
        Ok(row_id) => info!("🧾 已持久化 Gemini 分析日志 (ID: {}).", row_id),
        Err(err) => warn!("⚠️ 保存 Gemini 分析日志失败: {}", err),
    }

    Ok(())
}

fn has_tpsl_orders_for_position(orders: &[&OpenOrder], position_side: &str) -> (bool, bool) {
    let mut has_tp = false;
    let mut has_sl = false;

    for order in orders {
        let Some(side) = order.position_side.as_deref() else {
            continue;
        };
        if !side.eq_ignore_ascii_case(position_side) {
            continue;
        }

        let order_type = order.order_type.to_ascii_uppercase();
        if order_type.contains("TAKE_PROFIT") {
            has_tp = true;
        }
        if order_type.contains("STOP") {
            has_sl = true;
        }
    }

    (has_tp, has_sl)
}

async fn fulfill_pending_tpsl_orders(
    binance: &BinanceClient,
    position: &Position,
    pending: &PendingTpSlRecord,
    need_take_profit: bool,
    need_stop_loss: bool,
) -> Result<()> {
    if !need_take_profit && !need_stop_loss {
        return Ok(());
    }

    let mut quantity = position.size.abs();
    if quantity <= f64::EPSILON {
        quantity = pending.quantity;
    }

    if quantity <= f64::EPSILON {
        return Err(anyhow!("待设置止盈止损的数量无效"));
    }

    let side = pending.position_side.as_str();

    if need_take_profit {
        binance
            .set_take_profit(SYMBOL, side, quantity, pending.take_profit, None)
            .await?;
    }

    if need_stop_loss {
        binance
            .set_stop_loss(SYMBOL, side, quantity, pending.stop_loss, None)
            .await?;
    }

    Ok(())
}

fn convert_raw_klines(raw: Vec<Vec<f64>>) -> Vec<Kline> {
    raw.into_iter()
        .map(|values| Kline {
            timestamp: values.get(0).copied().unwrap_or_default() as i64,
            open: values.get(1).copied().unwrap_or_default(),
            high: values.get(2).copied().unwrap_or_default(),
            low: values.get(3).copied().unwrap_or_default(),
            close: values.get(4).copied().unwrap_or_default(),
            volume: values.get(5).copied().unwrap_or_default(),
        })
        .collect()
}

fn build_analysis_prompt(
    klines_1m: &[Kline],
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    klines_4h: &[Kline],
    indicators: &serde_json::Value,
    position: Option<&Position>,
    tpsl_orders: &[&OpenOrder],
    trigger_orders: &[&OpenOrder],
) -> String {
    // 格式化K线数据
    let format_klines = |klines: &[Kline], interval: &str| -> String {
        let mut result = format!(
            "\n=== {} K线 (最近{}根) ===\n",
            interval,
            klines.len().min(20)
        );
        for k in klines.iter().rev().take(20) {
            result.push_str(&format!(
                "{}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}\n",
                k.timestamp, k.open, k.high, k.low, k.close, k.volume
            ));
        }
        result
    };

    let all_klines_string = format!(
        "{}{}{}{}{}",
        format_klines(klines_1m, "1分钟"),
        format_klines(klines_5m, "5分钟"),
        format_klines(klines_15m, "15分钟"),
        format_klines(klines_1h, "1小时"),
        format_klines(klines_4h, "4小时")
    );

    let indicators_string = serde_json::to_string_pretty(indicators).unwrap_or_default();
    let position_string = position
        .map(|p| serde_json::to_string_pretty(p).unwrap_or_default())
        .unwrap_or_else(|| "无持仓".to_string());
    // 当前持仓尚未提供入场时间，暂以提示文本告知后续需要补全该字段
    let position_duration_text = if position.is_some() {
        "\n持仓时长: 建议在数据中添加入场时间字段以精确计算".to_string()
    } else {
        String::new()
    };
    let tpsl_string =
        serde_json::to_string_pretty(&tpsl_orders).unwrap_or_else(|_| "无止盈止损订单".to_string());
    let trigger_orders_display = if !trigger_orders.is_empty() {
        let orders: Vec<String> = trigger_orders
            .iter()
            .map(|o| {
                json!({
                    "orderId": o.order_id.as_str(),
                    "symbol": o.symbol.as_str(),
                    "side": o.side.as_deref(),
                    "positionSide": o.position_side.as_deref(),
                    "type": o.order_type.as_str(),
                    "triggerPrice": o.stop_price.unwrap_or(0.0),
                    "quantity": o.quantity.unwrap_or(0.0),
                    "status": o.status.as_str(),
                })
                .to_string()
            })
            .collect();
        format!("[\n    {}\n]", orders.join(",\n    "))
    } else {
        "[]".to_string()
    };

    format!(
        r#"你是一位顶尖的加密货币交易分析师。请结合以下最新的市场数据、技术指标、当前持仓信息、账户总览信息以及止盈止损订单信息，为 ETH-USDT 合约提供一个详细的交易策略。

**💰 资金配置说明:**
- 单次开仓本金: 动态调整（基于上次盈利，2-5 USDT）
- 杠杆倍数: 50倍
- 仓位模式: 逐仓 (ISOLATED)

**⚠️ 重要风控要求:**
1. **止损合适且损失较小**: 止损必须设置在合理位置，避免过大亏损
2. **优化进出场时机**: 目标是在最优的时间内赚取最多利润
3. **止盈止损方向示例**: 
   **做多(LONG)**:
   - ✅ 正确: 入场3400, 止盈3450 (高于入场), 止损3380 (低于入场)
   - ❌ 错误: 入场3400, 止盈3380 (低于入场), 止损3450 (高于入场) ← 方向反了！

   **做空(SHORT)**:
   - ✅ 正确: 入场3400, 止盈3350 (低于入场), 止损3420 (高于入场)
   - ❌ 错误: 入场3400, 止盈3450 (高于入场), 止损3380 (低于入场) ← 方向反了！

   **原则**: 
   - 做多时，价格上涨获利，止盈要高于入场价，止损要低于入场价
   - 做空时，价格下跌获利，止盈要低于入场价，止损要高于入场价
   - 止损点数要小于止盈点数，确保盈亏比合理
4. **逐仓风控**: 每个仓位独立，最大损失为本次本金，不会影响其他资金
5. **杠杆风险**: 50倍杠杆下，0.6%的反向波动即触发止损，必须设置合理的止损位

**1. K线数据 (多时间周期):**
每行格式: UTC时间, 开盘价, 最高价, 最低价, 收盘价, 成交量
{}

**2. 技术指标数据:**
```json
{}
```

**3. 当前持仓信息:**
```json
{}
```
{}

**4. 止盈止损订单当前委托:**
```json
{}
```

**5. 计划委托当前委托:**
```json
{}
```

**分析要求:**
请严格按照以下结构进行分析和输出：

1.  **市场趋势判断**:
    *   **综合判断**: 结合 K 线形态、成交量和所有技术指标（EMA, RSI, MACD, Bollinger Bands, Stochastic, ADX, ATR），明确判断当前市场的主要趋势是 **上涨**、**下跌** 还是 **震荡**。
    *   **判断信心度**: 以百分比形式给出你对趋势判断的信心度 (例如: 信心度: 85%)。
    *   **关键指标解读**: 简要说明几个关键指标（例如 MACD 的金叉/死叉，RSI 的超买/超卖区域，ADX 的趋势强度）是如何支持你的趋势判断的。

2.  **关键价位识别**:
    *   **支撑位**: 识别出 1-2 个最关键的短期支撑位。
    *   **压力位**: 识别出 1-2 个最关键的短期压力位。

3.  **交易策略与操作建议**:
    *   **基本原则**: 只有在市场出现明确的 **上涨** 或 **下跌** 趋势时才进行操作。如果判断为 **震荡** 或趋势不明朗，则首选 **保持观望 (Wait)**。
    *   **操作方向**: 明确建议 **做多 (Long)**、**做空 (Short)** 或 **保持观望 (Wait)**。
    *   **入场点位 (Entry Point)**: 如果建议操作，推荐一个具体的、可操作的明确入场价格点位。如果建议观望，则此处写"无"。
    *   **止盈位 (Take Profit)**: 如果建议操作，推荐一个明确的止盈价格。如果建议观望，则此处写"无"。
    *   **止损位 (Stop Loss)**: 如果建议操作，推荐一个明确的止损价格。如果建议观望，则此处写"无"。
    *   **持仓调整建议**: 根据当前持仓，给出相应的调整建议（例如：减仓、加仓、平仓等）。

4.  **最终操作建议**:

    根据以上所有信息，从以下6个操作中选择 **一个或多个** 最应该执行的操作，并按执行的先后顺序列出。

    **可选操作清单**:
    1.  **立即平仓** - 当前持仓风险过高或趋势明确反转时
    2.  **合约限价单下单** - 当前价格合适，直接开仓
    3.  **合约计划委托下单** - 预期突破关键价位，挂单等待触发
    4.  **合约计划委托撤单** - 之前的计划委托条件不再合理
    5.  **对仓位设置止盈止损订单** - 现有仓位缺少或需调整止盈止损
    6.  **止盈止损订单撤单** - 之前的止盈止损不符合当前市场

    **⚠️ 严格响应格式 (必须遵守)**:

    在你的分析最后，**必须**用以下格式输出最终操作建议，每行一个操作，按顺序编号:

    ```
    最终操作建议:
    1. [操作名称] [参数]
    ```

    **📋 各操作类型的格式要求**:

    **操作1: 立即平仓**
    ```
    最终操作建议:
    1. 立即平仓
    ```

    **操作2: 合约限价单下单**
    ```
    最终操作建议:
    1. 合约限价单下单 做多 开仓价格3200 止盈3250 止损3180
    ```
    或
    ```
    最终操作建议:
    1. 合约限价单下单 做空 开仓价格3240 止盈3210 止损3255
    ```

    **操作3: 合约计划委托下单**
    ```
    最终操作建议:
    1. 合约计划委托下单 做多 触发价格3200 止盈3250 止损3180
    ```
    或
    ```
    最终操作建议:
    1. 合约计划委托下单 做空 触发价格3250 止盈3220 止损3270
    ```

    **操作4: 合约计划委托撤单**
    ```
    最终操作建议:
    1. 合约计划委托撤单
    ```

    **操作5: 对仓位设置止盈止损订单**
    ```
    最终操作建议:
    1. 对仓位设置止盈止损订单 止盈3250 止损3180
    ```

    **操作6: 止盈止损订单撤单**
    ```
    最终操作建议:
    1. 止盈止损订单撤单
    ```

    **观望(不操作)**
    ```
    最终操作建议:
    观望
    ```

    **✅ 格式规则 (严格遵守)**:
    - 操作名称必须与上述6种**完全一致**
    - 方向必须是"做多"或"做空"
    - 价格必须是**纯数字**，不要加"$"、"USDT"等符号
    - 价格关键词: "开仓价格"、"触发价格"、"止盈"、"止损"
    - 做多: 止盈 > 开仓价/触发价 > 止损
    - 做空: 止损 > 开仓价/触发价 > 止盈
    - 可以输出多个操作，每行一个，按序号编号
    - 不操作时输出"观望"

请确保你的分析逻辑清晰、依据充分，并直接给出最终的操作建议。"#,
        all_klines_string,
        indicators_string,
        position_string,
        position_duration_text,
        tpsl_string,
        trigger_orders_display
    )
}

async fn calculate_dynamic_capital(db: &Database) -> f64 {
    match db.get_last_profit(SYMBOL) {
        Ok(Some(last_profit)) => {
            let capital = if last_profit < 0.0 {
                MIN_DYNAMIC_CAPITAL
            } else if last_profit < MIN_DYNAMIC_CAPITAL {
                MIN_DYNAMIC_CAPITAL
            } else if last_profit > MAX_DYNAMIC_CAPITAL {
                MAX_DYNAMIC_CAPITAL
            } else {
                last_profit
            };
            info!(
                "📈 最近一次盈亏: {:.2} USDT，动态本金设置为 {:.2} USDT",
                last_profit, capital
            );
            capital
        }
        Ok(None) => {
            info!(
                "ℹ️ 暂无历史盈亏记录，使用默认 {:.2} USDT 本金。",
                MIN_DYNAMIC_CAPITAL
            );
            MIN_DYNAMIC_CAPITAL
        }
        Err(err) => {
            warn!(
                "⚠️ 查询历史盈亏失败: {}，回退使用默认 {:.2} USDT 本金。",
                err, MIN_DYNAMIC_CAPITAL
            );
            MIN_DYNAMIC_CAPITAL
        }
    }
}

fn calculate_indicators(klines: &[Kline]) -> Result<serde_json::Value> {
    if klines.is_empty() {
        return Ok(json!({}));
    }

    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
    let latest_close = closes.last().copied().unwrap_or(0.0);

    // 计算EMA
    let ema_20 = calculate_ema(&closes, 20);
    let ema_50 = calculate_ema(&closes, 50);
    let ema_200 = calculate_ema(&closes, 200);

    // 计算RSI
    let rsi_14 = calculate_rsi(&closes, 14);

    // 计算MACD
    let (macd_line, signal_line, histogram) = calculate_macd(&closes);

    // 计算布林带
    let (bb_upper, bb_middle, bb_lower) = calculate_bollinger_bands(&closes, 20, 2.0);

    // 计算ATR
    let atr_14 = calculate_atr(&highs, &lows, &closes, 14);

    Ok(json!({
        "current_price": latest_close,
        "EMA": {
            "ema_20": ema_20,
            "ema_50": ema_50,
            "ema_200": ema_200,
            "trend": if ema_20 > ema_50 { "上涨" } else { "下跌" }
        },
        "RSI": {
            "rsi_14": rsi_14,
            "status": if rsi_14 > 70.0 { "超买" } else if rsi_14 < 30.0 { "超卖" } else { "中性" }
        },
        "MACD": {
            "macd_line": macd_line,
            "signal_line": signal_line,
            "histogram": histogram,
            "trend": if histogram > 0.0 { "金叉" } else { "死叉" }
        },
        "Bollinger_Bands": {
            "upper": bb_upper,
            "middle": bb_middle,
            "lower": bb_lower,
            "position": if latest_close > bb_upper { "超买区" } else if latest_close < bb_lower { "超卖区" } else { "正常区" }
        },
        "ATR": {
            "atr_14": atr_14,
            "volatility": if atr_14 / latest_close > 0.02 { "高" } else { "低" }
        }
    }))
}

fn calculate_ema(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return prices.last().copied().unwrap_or(0.0);
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = prices[..period].iter().sum::<f64>() / period as f64;

    for price in prices.iter().skip(period) {
        ema = (price - ema) * multiplier + ema;
    }

    ema
}

fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period + 1 {
        return 50.0;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses += change.abs();
        }
    }

    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn calculate_macd(prices: &[f64]) -> (f64, f64, f64) {
    if prices.len() < 26 {
        return (0.0, 0.0, 0.0);
    }

    let mut macd_history = Vec::with_capacity(prices.len() - 25);
    for i in 26..=prices.len() {
        let window = &prices[..i];
        let ema_12 = calculate_ema(window, 12);
        let ema_26 = calculate_ema(window, 26);
        macd_history.push(ema_12 - ema_26);
    }

    let signal_line = calculate_ema(&macd_history, 9);
    let macd_line = macd_history.last().copied().unwrap_or(0.0);
    let histogram = macd_line - signal_line;

    (macd_line, signal_line, histogram)
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, std_dev: f64) -> (f64, f64, f64) {
    if prices.len() < period {
        let last = prices.last().copied().unwrap_or(0.0);
        return (last, last, last);
    }

    let recent_prices = &prices[prices.len() - period..];
    let middle = recent_prices.iter().sum::<f64>() / period as f64;

    let variance = recent_prices
        .iter()
        .map(|p| (p - middle).powi(2))
        .sum::<f64>()
        / period as f64;
    let std = variance.sqrt();

    let upper = middle + (std * std_dev);
    let lower = middle - (std * std_dev);

    (upper, middle, lower)
}

fn calculate_atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> f64 {
    if highs.len() < period + 1 {
        return 0.0;
    }

    let mut true_ranges = Vec::new();
    for i in 1..highs.len() {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        true_ranges.push(tr);
    }

    let recent_tr = &true_ranges[true_ranges.len().saturating_sub(period)..];
    recent_tr.iter().sum::<f64>() / recent_tr.len() as f64
}

async fn execute_trade_action(
    binance: &BinanceClient,
    analysis_text: &str,
    current_position: Option<Position>,
    db: &Database,
    capital: f64,
) -> Result<()> {
    let Some(signal) = parse_structured_actions(analysis_text) else {
        info!("⚪ Gemini未给出可执行的交易信号，跳过自动下单。");
        return Ok(());
    };

    let mut has_position = current_position.is_some();

    for action in signal.actions {
        match action {
            ParsedAction::ClosePosition => {
                if !has_position {
                    warn!("⚠️ 当前无持仓，无需平仓。");
                    continue;
                }

                let positions = binance.get_positions().await?;
                let eth_position = positions
                    .iter()
                    .find(|p| p.symbol == SYMBOL && p.size.abs() > f64::EPSILON);

                if let Some(pos) = eth_position {
                    let (position_side, close_side) = if pos.side.eq_ignore_ascii_case("LONG") {
                        ("LONG", "SELL")
                    } else if pos.side.eq_ignore_ascii_case("SHORT") {
                        ("SHORT", "BUY")
                    } else {
                        (pos.side.as_str(), "SELL")
                    };

                    info!(
                        "📝 准备平仓: {} 仓位 {:.6} @ ${:.2}",
                        position_side,
                        pos.size.abs(),
                        pos.entry_price
                    );

                    binance
                        .close_position(SYMBOL, close_side, pos.size.abs())
                        .await?;

                    let exit_price = match binance.get_current_price(SYMBOL).await {
                        Ok(price) if price > 0.0 => price,
                        Ok(_) => pos.entry_price,
                        Err(err) => {
                            warn!("⚠️ 获取平仓价格失败: {}，使用入场价代替。", err);
                            pos.entry_price
                        }
                    };

                    info!("✅ 持仓已平仓");
                    let profit = pos.pnl;
                    let exit_time = chrono::Utc::now().timestamp();
                    if let Err(e) = db.record_trade_profit(
                        SYMBOL,
                        0,
                        exit_time,
                        pos.entry_price,
                        exit_price,
                        pos.size.abs(),
                        position_side,
                        profit,
                        capital,
                    ) {
                        warn!("⚠️ 记录交易盈利失败: {}", e);
                    }
                    has_position = false;
                } else {
                    warn!("⚠️ 未找到ETH持仓，跳过平仓。");
                }
            }
            ParsedAction::CancelTPSL => {
                let all_orders = binance.get_open_orders(Some(SYMBOL)).await?;
                let tpsl_orders: Vec<_> = all_orders
                    .iter()
                    .filter(|o| {
                        o.order_type == "STOP_MARKET"
                            || o.order_type == "TAKE_PROFIT_MARKET"
                            || o.order_type == "STOP"
                            || o.order_type == "TAKE_PROFIT"
                    })
                    .collect();

                if tpsl_orders.is_empty() {
                    info!("ℹ️ 无止盈止损订单需要撤销。");
                    continue;
                }

                info!("📝 准备撤销 {} 个止盈止损订单", tpsl_orders.len());

                for order in tpsl_orders {
                    match binance.cancel_order(SYMBOL, &order.order_id).await {
                        Ok(_) => info!(
                            "✅ 已撤销订单: {} (类型: {})",
                            order.order_id, order.order_type
                        ),
                        Err(e) => warn!("⚠️ 撤销订单失败 {}: {}", order.order_id, e),
                    }
                }
            }
            ParsedAction::CancelTriggerOrder => {
                let all_orders = binance.get_open_orders(Some(SYMBOL)).await?;
                let trigger_orders: Vec<_> = all_orders
                    .iter()
                    .filter(|o| {
                        (o.order_type.contains("STOP") && o.order_type != "STOP_MARKET")
                            || (o.order_type.contains("TAKE_PROFIT")
                                && o.order_type != "TAKE_PROFIT_MARKET")
                            || o.order_type == "STOP"
                            || o.order_type == "TAKE_PROFIT"
                    })
                    .collect();

                if trigger_orders.is_empty() {
                    info!("ℹ️ 无计划委托订单需要撤销。");
                    continue;
                }

                info!("📝 准备撤销 {} 个计划委托订单", trigger_orders.len());

                for order in trigger_orders {
                    match binance.cancel_order(SYMBOL, &order.order_id).await {
                        Ok(_) => info!(
                            "✅ 已撤销计划委托: {} (类型: {})",
                            order.order_id, order.order_type
                        ),
                        Err(e) => warn!("⚠️ 撤销订单失败 {}: {}", order.order_id, e),
                    }
                }
            }
            ParsedAction::SetTPSL {
                take_profit,
                stop_loss,
            } => {
                if !has_position {
                    warn!("⚠️ 当前无持仓，无法设置止盈止损。");
                    continue;
                }
                if take_profit <= f64::EPSILON || stop_loss <= f64::EPSILON {
                    warn!("⚠️ 止盈/止损价格无效，跳过设置命令。");
                    continue;
                }

                let positions = binance.get_positions().await?;
                let Some(pos) = positions
                    .iter()
                    .find(|p| p.symbol == SYMBOL && p.size.abs() > f64::EPSILON)
                else {
                    warn!("⚠️ 未找到ETH持仓，无法设置止盈止损。");
                    continue;
                };

                let quantity = pos.size.abs();
                if quantity <= f64::EPSILON {
                    warn!("⚠️ 当前持仓数量无效，无法设置止盈止损。");
                    continue;
                }

                let side_str = if pos.side.eq_ignore_ascii_case("SHORT") {
                    "SHORT"
                } else {
                    "LONG"
                };

                binance
                    .set_take_profit(SYMBOL, side_str, quantity, take_profit, None)
                    .await?;
                binance
                    .set_stop_loss(SYMBOL, side_str, quantity, stop_loss, None)
                    .await?;

                info!(
                    "✅ 已为 {} 仓位设置止盈 {:.2} / 止损 {:.2}",
                    side_str, take_profit, stop_loss
                );
            }
            ParsedAction::LimitOrder(limit) => {
                if has_position {
                    warn!("⚠️ 当前已有持仓或挂单，出于风控不再开新仓。");
                    continue;
                }

                let Some(direction) = limit.direction else {
                    warn!("⚠️ 无法识别做多/做空方向，跳过限价单执行。");
                    continue;
                };
                let Some(entry_price) = limit.entry_price.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析入场价格，跳过限价单执行。");
                    continue;
                };
                let Some(take_profit) = limit.take_profit.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析止盈价格，跳过限价单执行。");
                    continue;
                };
                let Some(stop_loss) = limit.stop_loss.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析止损价格，跳过限价单执行。");
                    continue;
                };

                let current_price = binance.get_current_price(SYMBOL).await?;
                if current_price <= f64::EPSILON {
                    warn!("⚠️ 当前行情价格无效，跳过限价单执行。");
                    continue;
                }
                let price_deviation = ((entry_price - current_price) / current_price).abs();
                if price_deviation > 0.15 {
                    warn!(
                        "⚠️ 入场价 {:.2} 偏离当前价 {:.2} 超过 15% ({:.1}%)，可能存在解析错误，跳过执行。",
                        entry_price,
                        current_price,
                        price_deviation * 100.0
                    );
                    continue;
                }

                let tp_sl_invalid = match direction {
                    PositionSide::Long => take_profit <= entry_price || stop_loss >= entry_price,
                    PositionSide::Short => take_profit >= entry_price || stop_loss <= entry_price,
                };
                if tp_sl_invalid {
                    warn!("⚠️ 止盈/止损与入场价关系不合理，跳过限价单执行。");
                    continue;
                }

                let account_info = binance.get_account_info().await?;
                let available_balance = account_info
                    .availableBalance
                    .parse::<f64>()
                    .unwrap_or_default();
                if available_balance + f64::EPSILON < capital {
                    warn!(
                        "⚠️ 可用余额 {:.2} USDT 小于策略本金 {:.2} USDT，跳过开仓。",
                        available_balance, capital
                    );
                    continue;
                }

                let rules = binance.get_symbol_trading_rules(SYMBOL).await?;
                let quantity = binance.calculate_quantity_with_margin(
                    entry_price,
                    capital,
                    LEVERAGE,
                    &rules,
                )?;
                if !quantity.is_finite() || quantity <= 0.0 {
                    warn!("⚠️ 计算得到的下单数量无效 ({:.6})，取消执行。", quantity);
                    continue;
                }

                info!(
                    "📝 准备执行限价单: {:?} 入场 {:.2} 止盈 {:.2} 止损 {:.2} 数量 {:.6}",
                    direction, entry_price, take_profit, stop_loss, quantity
                );

                // 逐仓模式保证单笔仓位独立风险
                binance.set_margin_type(SYMBOL, "ISOLATED").await?;
                binance.set_leverage(SYMBOL, LEVERAGE).await?;
                let order_side = direction.order_side();
                let position_side = direction.as_position_str();

                binance
                    .limit_order(
                        SYMBOL,
                        quantity,
                        order_side,
                        entry_price,
                        Some(position_side),
                        false,
                    )
                    .await?;
                binance
                    .set_take_profit(SYMBOL, position_side, quantity, take_profit, None)
                    .await?;
                binance
                    .set_stop_loss(SYMBOL, position_side, quantity, stop_loss, None)
                    .await?;

                info!("✅ Gemini限价单及止盈/止损指令已提交。");
                has_position = true;
            }
            ParsedAction::TriggerOrder(trigger) => {
                if has_position {
                    warn!("⚠️ 当前已有持仓或挂单，出于风控不再开新仓。");
                    continue;
                }

                let Some(direction) = trigger.direction else {
                    warn!("⚠️ 无法识别做多/做空方向，跳过计划委托执行。");
                    continue;
                };
                let Some(trigger_price) = trigger.trigger_price.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析触发价格，跳过计划委托执行。");
                    continue;
                };
                let Some(take_profit) = trigger.take_profit.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析止盈价格，跳过计划委托执行。");
                    continue;
                };
                let Some(stop_loss) = trigger.stop_loss.filter(|p| *p > 0.0) else {
                    warn!("⚠️ 无法解析止损价格，跳过计划委托执行。");
                    continue;
                };

                let current_price = binance.get_current_price(SYMBOL).await?;
                if current_price <= f64::EPSILON {
                    warn!("⚠️ 当前行情价格无效，跳过计划委托执行。");
                    continue;
                }

                let reference_entry = trigger
                    .limit_price
                    .filter(|p| *p > 0.0)
                    .unwrap_or(trigger_price);
                let price_deviation = ((reference_entry - current_price) / current_price).abs();
                if price_deviation > 0.15 {
                    warn!(
                        "⚠️ 计划委托参考价 {:.2} 与当前价 {:.2} 偏差 {:.1}%，可能存在解析错误，跳过执行。",
                        reference_entry,
                        current_price,
                        price_deviation * 100.0
                    );
                    continue;
                }

                let tp_sl_invalid = match direction {
                    PositionSide::Long => {
                        take_profit <= reference_entry || stop_loss >= reference_entry
                    }
                    PositionSide::Short => {
                        take_profit >= reference_entry || stop_loss <= reference_entry
                    }
                };
                if tp_sl_invalid {
                    warn!("⚠️ 止盈/止损与计划委托价格关系不合理，跳过执行。");
                    continue;
                }

                let account_info = binance.get_account_info().await?;
                let available_balance = account_info
                    .availableBalance
                    .parse::<f64>()
                    .unwrap_or_default();
                if available_balance + f64::EPSILON < capital {
                    warn!(
                        "⚠️ 可用余额 {:.2} USDT 小于策略本金 {:.2} USDT，跳过开仓。",
                        available_balance, capital
                    );
                    continue;
                }

                let trigger_orders: Vec<_> = binance
                    .get_open_orders(Some(SYMBOL))
                    .await?
                    .into_iter()
                    .filter(|o| {
                        (o.order_type == "STOP" || o.order_type == "TAKE_PROFIT")
                            && o.status == "NEW"
                    })
                    .collect();

                let has_same_direction_trigger = trigger_orders.iter().any(|o| {
                    let order_is_long = o.position_side.as_deref() == Some("LONG");
                    let order_is_short = o.position_side.as_deref() == Some("SHORT");

                    match direction {
                        PositionSide::Long => order_is_long,
                        PositionSide::Short => order_is_short,
                    }
                });

                if has_same_direction_trigger {
                    warn!("⚠️ 已存在同方向触发单,跳过本次计划委托以避免风险叠加。");
                    let existing_orders: Vec<String> = trigger_orders
                        .iter()
                        .filter(|o| {
                            let order_is_long = o.position_side.as_deref() == Some("LONG");
                            let order_is_short = o.position_side.as_deref() == Some("SHORT");
                            match direction {
                                PositionSide::Long => order_is_long,
                                PositionSide::Short => order_is_short,
                            }
                        })
                        .map(|o| {
                            format!(
                                "订单ID: {}, 触发价: {}",
                                o.order_id,
                                o.stop_price.unwrap_or(0.0)
                            )
                        })
                        .collect();
                    warn!("   现有同方向触发单: {:?}", existing_orders);
                    continue;
                }

                // 价格方向合法性检查
                let current_price = binance.get_current_price(SYMBOL).await?;

                let is_valid_trigger_price = match direction {
                    PositionSide::Long => {
                        // 做多STOP触发单: 触发价应该 < 当前价 (跌破时入场)
                        //             或  触发价 > 当前价 (突破时入场)
                        // Binance允许两种情况,不做限制
                        true
                    }
                    PositionSide::Short => {
                        // 做空STOP触发单: 触发价应该 < 当前价 (跌破时入场)
                        // 如果触发价 > 当前价,会触发 -2021 错误
                        trigger_price < current_price
                    }
                };

                if !is_valid_trigger_price {
                    warn!(
                        "⚠️ 触发单价格方向不合法: {:?} 触发价 {:.2} vs 当前价 {:.2}",
                        direction, trigger_price, current_price
                    );
                    warn!(
                        "   做空触发单要求: 触发价 < 当前价 (等待跌破入场)。当前触发价高于市场价,会立即触发。"
                    );
                    warn!(
                        "   建议: 使用限价单在 {:.2} 挂单等待反弹,或调整触发价至 {:.2} 以下",
                        trigger_price,
                        current_price - 1.0
                    );
                    continue;
                }

                let rules = binance.get_symbol_trading_rules(SYMBOL).await?;

                // 触发单类型: 开仓场景统一使用 STOP，避免被 Binance 识别为止盈单
                let trigger_type = "STOP";

                let quantity = binance.calculate_quantity_with_margin(
                    trigger_price,
                    capital,
                    LEVERAGE,
                    &rules,
                )?;
                if !quantity.is_finite() || quantity <= 0.0 {
                    warn!("⚠️ 计算得到的下单数量无效 ({:.6})，取消执行。", quantity);
                    continue;
                }

                info!(
                    "📝 准备执行计划委托: {:?} 触发价 {:.2} 止盈 {:.2} 止损 {:.2} 数量 {:.6} 类型 {}",
                    direction, trigger_price, take_profit, stop_loss, quantity, trigger_type
                );

                // 逐仓模式保证单笔仓位独立风险
                binance.set_margin_type(SYMBOL, "ISOLATED").await?;
                binance.set_leverage(SYMBOL, LEVERAGE).await?;
                let position_side = direction.as_position_str();

                let order_id = binance
                    .place_trigger_order(
                        SYMBOL,
                        trigger_type,
                        "OPEN",
                        position_side,
                        quantity,
                        trigger_price,
                        Some(trigger_price),
                    )
                    .await?;

                info!(
                    "✅ Gemini计划委托指令已提交，开始轮询成交状态 (订单ID: {}).",
                    order_id
                );

                let mut order_filled = false;
                for attempt in 1..=TRIGGER_ORDER_MAX_WAIT_ATTEMPTS {
                    match binance.get_order_status(SYMBOL, &order_id).await {
                        Ok(status_text) => match status_text.as_str() {
                            "FILLED" => {
                                info!(
                                    "🎯 触发单已成交，准备立即设置止盈止损 (订单ID: {}).",
                                    order_id
                                );
                                order_filled = true;
                                break;
                            }
                            "NEW" | "PARTIALLY_FILLED" => {
                                info!(
                                    "⏳ 等待触发单成交 (状态: {} 尝试 {}/{})",
                                    status_text, attempt, TRIGGER_ORDER_MAX_WAIT_ATTEMPTS
                                );
                            }
                            other_status => {
                                warn!(
                                    "⚠️ 触发单进入异常状态: {} (订单ID: {})",
                                    other_status, order_id
                                );
                                break;
                            }
                        },
                        Err(e) => {
                            warn!(
                                "⚠️ 查询触发单状态失败 (尝试 {}/{}): {}",
                                attempt, TRIGGER_ORDER_MAX_WAIT_ATTEMPTS, e
                            );
                            break;
                        }
                    }

                    if order_filled {
                        break;
                    }

                    if attempt < TRIGGER_ORDER_MAX_WAIT_ATTEMPTS {
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            TRIGGER_ORDER_POLL_INTERVAL_SECS,
                        ))
                        .await;
                    }
                }

                let mut pending_needed = false;
                let mut pending_quantity = quantity;

                if order_filled {
                    match binance.get_positions().await {
                        Ok(positions) => {
                            let maybe_position = positions.iter().find(|p| {
                                p.symbol == SYMBOL && p.side.eq_ignore_ascii_case(position_side)
                            });

                            if let Some(pos) = maybe_position {
                                let actual_quantity = pos.size.abs();
                                if actual_quantity <= f64::EPSILON {
                                    warn!("⚠️ 查询到的持仓数量为 0，改为登记待补设任务。");
                                    pending_needed = true;
                                } else {
                                    pending_quantity = actual_quantity;
                                    let mut tp_set = false;
                                    let mut sl_set = false;

                                    match binance
                                        .set_take_profit(
                                            SYMBOL,
                                            position_side,
                                            actual_quantity,
                                            take_profit,
                                            None,
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            info!("✅ 止盈单已设置: {:.2}", take_profit);
                                            tp_set = true;
                                        }
                                        Err(e) => {
                                            error!("❌ 止盈单设置失败: {}", e);
                                            pending_needed = true;
                                        }
                                    }

                                    match binance
                                        .set_stop_loss(
                                            SYMBOL,
                                            position_side,
                                            actual_quantity,
                                            stop_loss,
                                            None,
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            info!("✅ 止损单已设置: {:.2}", stop_loss);
                                            sl_set = true;
                                        }
                                        Err(e) => {
                                            let err_msg = e.to_string();
                                            if err_msg.contains("-2021") {
                                                match binance.get_current_price(SYMBOL).await {
                                                    Ok(latest_price) => {
                                                        let adjusted_stop = if matches!(
                                                            direction,
                                                            PositionSide::Short
                                                        ) {
                                                            latest_price + STOP_LOSS_RETRY_OFFSET
                                                        } else {
                                                            (latest_price - STOP_LOSS_RETRY_OFFSET)
                                                                .max(1.0)
                                                        };
                                                        warn!(
                                                            "⚠️ 止损价过近 (-2021)，调整为 {:.2} 重新提交。",
                                                            adjusted_stop
                                                        );
                                                        match binance
                                                            .set_stop_loss(
                                                                SYMBOL,
                                                                position_side,
                                                                actual_quantity,
                                                                adjusted_stop,
                                                                None,
                                                            )
                                                            .await
                                                        {
                                                            Ok(_) => {
                                                                info!(
                                                                    "✅ 止损单已设置(调整后): {:.2}",
                                                                    adjusted_stop
                                                                );
                                                                sl_set = true;
                                                            }
                                                            Err(adj_err) => {
                                                                error!(
                                                                    "❌ 止损单设置失败(调整后): {}",
                                                                    adj_err
                                                                );
                                                                pending_needed = true;
                                                            }
                                                        }
                                                    }
                                                    Err(price_err) => {
                                                        error!(
                                                            "❌ 获取当前价格失败，无法调整止损: {}",
                                                            price_err
                                                        );
                                                        pending_needed = true;
                                                    }
                                                }
                                            } else {
                                                error!("❌ 止损单设置失败: {}", err_msg);
                                                pending_needed = true;
                                            }
                                        }
                                    }

                                    if tp_set && sl_set {
                                        info!("🛡️ 触发单成交后已完成止盈止损设置。");
                                    } else {
                                        warn!("⚠️ 止盈/止损存在未成功设置的条目，已登记回退任务。");
                                    }
                                }
                            } else {
                                warn!("⚠️ 未找到对应持仓，改为登记待补设任务。");
                                pending_needed = true;
                            }
                        }
                        Err(e) => {
                            warn!("⚠️ 查询持仓信息失败，改为登记待补设任务: {}", e);
                            pending_needed = true;
                        }
                    }
                } else {
                    let total_wait_secs =
                        TRIGGER_ORDER_POLL_INTERVAL_SECS * TRIGGER_ORDER_MAX_WAIT_ATTEMPTS as u64;
                    warn!(
                        "⚠️ 触发单在 {} 秒内未成交，将止盈止损登记到待补设队列。",
                        total_wait_secs
                    );
                    pending_needed = true;
                }

                if pending_needed {
                    match db.enqueue_pending_tpsl(
                        SYMBOL,
                        position_side,
                        pending_quantity,
                        take_profit,
                        stop_loss,
                    ) {
                        Ok(record_id) => {
                            info!("🧾 已登记待设置止盈止损任务 (记录ID: {}).", record_id)
                        }
                        Err(e) => warn!("⚠️ 记录待设置止盈止损信息失败: {}", e),
                    }
                }
                has_position = true;
            }
        }
    }

    Ok(())
}

fn actions_to_json(actions: &[ParsedAction]) -> serde_json::Value {
    serde_json::Value::Array(
        actions
            .iter()
            .map(|action| match action {
                ParsedAction::LimitOrder(signal) => json!({
                    "type": "limit_order",
                    "direction": signal.direction.map(|d| d.as_position_str()),
                    "entry_price": signal.entry_price,
                    "take_profit": signal.take_profit,
                    "stop_loss": signal.stop_loss,
                }),
                ParsedAction::TriggerOrder(signal) => json!({
                    "type": "trigger_order",
                    "direction": signal.direction.map(|d| d.as_position_str()),
                    "trigger_price": signal.trigger_price,
                    "limit_price": signal.limit_price,
                    "take_profit": signal.take_profit,
                    "stop_loss": signal.stop_loss,
                }),
                ParsedAction::ClosePosition => json!({ "type": "close_position" }),
                ParsedAction::CancelTPSL => json!({ "type": "cancel_tpsl" }),
                ParsedAction::CancelTriggerOrder => json!({ "type": "cancel_trigger_order" }),
                ParsedAction::SetTPSL {
                    take_profit,
                    stop_loss,
                } => json!({
                    "type": "set_tpsl",
                    "take_profit": take_profit,
                    "stop_loss": stop_loss,
                }),
            })
            .collect(),
    )
}

/// 解析Gemini返回的结构化操作建议
fn parse_structured_actions(analysis_text: &str) -> Option<TradingSignal> {
    let marker = "最终操作建议:";
    let start_idx = match analysis_text.find(marker) {
        Some(idx) => idx,
        None => {
            warn!("⚠️ 未找到“最终操作建议”段落，无法解析结构化动作。");
            return None;
        }
    };
    let section = &analysis_text[start_idx + marker.len()..];

    let mut actions = Vec::new();

    for raw_line in section.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let first_non_space = line.chars().find(|c| !c.is_whitespace());
        let Some(first_char) = first_non_space else {
            continue;
        };
        if !first_char.is_ascii_digit() {
            continue;
        }

        let content = line
            .trim_start_matches(|c: char| {
                c.is_ascii_digit() || matches!(c, '.' | '、' | ')' | '(' | ' ')
            })
            .trim();

        if content.is_empty() {
            continue;
        }

        if content.contains("观望") {
            info!("ℹ️ Gemini建议观望，未生成可执行信号。");
            return None;
        }

        if content.starts_with("立即平仓") {
            actions.push(ParsedAction::ClosePosition);
            continue;
        }

        if content.starts_with("合约计划委托撤单")
            || content.starts_with("撤销计划委托")
            || content.starts_with("取消计划委托")
        {
            actions.push(ParsedAction::CancelTriggerOrder);
            continue;
        }

        if content.starts_with("止盈止损订单撤单") || content.starts_with("撤销止盈止损")
        {
            actions.push(ParsedAction::CancelTPSL);
            continue;
        }

        if content.starts_with("对仓位设置止盈止损") || content.starts_with("设置止盈止损")
        {
            let Some(take_profit) = extract_price(content, &TP_KEYWORDS) else {
                warn!("⚠️ 无法解析止盈价格: {}", content);
                continue;
            };
            let Some(stop_loss) = extract_price(content, &SL_KEYWORDS) else {
                warn!("⚠️ 无法解析止损价格: {}", content);
                continue;
            };

            actions.push(ParsedAction::SetTPSL {
                take_profit,
                stop_loss,
            });
            continue;
        }

        if content.starts_with("合约限价单下单") || content.starts_with("限价单") {
            let Some(direction) = detect_direction(content) else {
                warn!("⚠️ 无法识别限价单方向: {}", content);
                continue;
            };
            let Some(entry_price) = extract_price(content, &ENTRY_KEYWORDS) else {
                warn!("⚠️ 无法解析限价单入场价: {}", content);
                continue;
            };
            let Some(take_profit) = extract_price(content, &TP_KEYWORDS) else {
                warn!("⚠️ 无法解析限价单止盈价: {}", content);
                continue;
            };
            let Some(stop_loss) = extract_price(content, &SL_KEYWORDS) else {
                warn!("⚠️ 无法解析限价单止损价: {}", content);
                continue;
            };

            actions.push(ParsedAction::LimitOrder(LimitOrderSignal {
                direction: Some(direction),
                entry_price: Some(entry_price),
                take_profit: Some(take_profit),
                stop_loss: Some(stop_loss),
            }));
            continue;
        }

        if content.starts_with("合约计划委托下单") || content.starts_with("计划委托") {
            let Some(direction) = detect_direction(content) else {
                warn!("⚠️ 无法识别计划委托方向: {}", content);
                continue;
            };
            let Some(trigger_price) = extract_price(content, &TRIGGER_KEYWORDS) else {
                warn!("⚠️ 无法解析触发价格: {}", content);
                continue;
            };
            let Some(take_profit) = extract_price(content, &TP_KEYWORDS) else {
                warn!("⚠️ 无法解析计划委托止盈价: {}", content);
                continue;
            };
            let Some(stop_loss) = extract_price(content, &SL_KEYWORDS) else {
                warn!("⚠️ 无法解析计划委托止损价: {}", content);
                continue;
            };

            actions.push(ParsedAction::TriggerOrder(TriggerOrderSignal {
                direction: Some(direction),
                trigger_price: Some(trigger_price),
                limit_price: None,
                take_profit: Some(take_profit),
                stop_loss: Some(stop_loss),
            }));
            continue;
        }
    }

    if actions.is_empty() {
        warn!("⚠️ 未能从结构化输出中解析到任何操作。");
        None
    } else {
        Some(TradingSignal { actions })
    }
}

/// 从文本中提取价格数字
fn extract_price(text: &str, keywords: &[&str]) -> Option<f64> {
    for keyword in keywords {
        if let Some(idx) = text.find(keyword) {
            let after = &text[idx + keyword.len()..];
            let price_str: String = after
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();

            if price_str.is_empty() {
                continue;
            }

            if let Ok(price) = price_str.parse::<f64>() {
                if price > 0.0 && price < 1_000_000.0 {
                    return Some(price);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
fn parse_trading_signal(analysis_text: &str) -> Option<TradingSignal> {
    let action_section = analysis_text
        .split("最终操作建议")
        .nth(1)
        .unwrap_or(analysis_text);

    let mut actions = Vec::new();
    for raw_line in action_section.lines() {
        let normalized = normalize_action_line(raw_line);
        if normalized.is_empty() {
            continue;
        }

        if matches_any_keyword(&normalized, LIMIT_ORDER_KEYWORDS) {
            actions.push(ParsedAction::LimitOrder(build_limit_order_signal(
                raw_line,
                action_section,
                analysis_text,
            )));
        } else if matches_any_keyword(&normalized, TRIGGER_ORDER_KEYWORDS) {
            actions.push(ParsedAction::TriggerOrder(parse_trigger_order_signal(
                raw_line,
                action_section,
                analysis_text,
            )));
        } else if matches_any_keyword(&normalized, CLOSE_KEYWORDS) {
            actions.push(ParsedAction::ClosePosition);
        } else if matches_any_keyword(&normalized, CANCEL_TPSL_KEYWORDS) {
            actions.push(ParsedAction::CancelTPSL);
        } else if matches_any_keyword(&normalized, CANCEL_TRIGGER_KEYWORDS) {
            actions.push(ParsedAction::CancelTriggerOrder);
        }
    }

    if actions.is_empty() {
        let lower_text = analysis_text.to_lowercase();
        let has_suggestion = ["建议", "操作", "signal"]
            .iter()
            .any(|kw| lower_text.contains(kw));
        if has_suggestion {
            warn!("⚠️ Gemini 似乎给出了建议但解析失败，建议检查输出格式。");
            let preview: String = analysis_text.chars().take(500).collect();
            info!("📄 分析文本片段: {}", preview);
        }
        None
    } else {
        Some(TradingSignal { actions })
    }
}

fn build_limit_order_signal(line: &str, section: &str, full_text: &str) -> LimitOrderSignal {
    let direction = detect_direction(line)
        .or_else(|| detect_direction(section))
        .or_else(|| detect_direction(full_text));

    let entry_price = extract_price_with_patterns(line, &ENTRY_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &ENTRY_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &ENTRY_KEYWORDS));
    let take_profit = extract_price_with_patterns(line, &TP_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &TP_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &TP_KEYWORDS));
    let stop_loss = extract_price_with_patterns(line, &SL_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &SL_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &SL_KEYWORDS));

    LimitOrderSignal {
        direction,
        entry_price,
        take_profit,
        stop_loss,
    }
}

fn parse_trigger_order_signal(line: &str, section: &str, full_text: &str) -> TriggerOrderSignal {
    let direction = detect_direction(line)
        .or_else(|| detect_direction(section))
        .or_else(|| detect_direction(full_text));

    let trigger_price = extract_price_with_patterns(line, &TRIGGER_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &TRIGGER_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &TRIGGER_KEYWORDS));
    let limit_price = extract_price_with_patterns(line, &LIMIT_PRICE_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &LIMIT_PRICE_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &LIMIT_PRICE_KEYWORDS));
    let take_profit = extract_price_with_patterns(line, &TP_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &TP_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &TP_KEYWORDS));
    let stop_loss = extract_price_with_patterns(line, &SL_KEYWORDS)
        .or_else(|| extract_price_with_patterns(section, &SL_KEYWORDS))
        .or_else(|| extract_price_with_patterns(full_text, &SL_KEYWORDS));

    TriggerOrderSignal {
        direction,
        trigger_price,
        limit_price,
        take_profit,
        stop_loss,
    }
}

const LIMIT_ORDER_KEYWORDS: &[&str] = &[
    "合约限价单",
    "限价单",
    "限价开仓",
    "limit order",
    "limit entry",
    "下限价单",
    "挂限价",
];
const TRIGGER_ORDER_KEYWORDS: &[&str] = &[
    "计划委托",
    "合约计划委托",
    "trigger order",
    "conditional order",
    "止损单",
    "触发单",
    "计划下单",
    "条件单",
];
const CLOSE_KEYWORDS: &[&str] = &[
    "立即平仓",
    "平仓",
    "close position",
    "close",
    "退出",
    "全部平仓",
    "止盈离场",
    "止损离场",
];
const CANCEL_TPSL_KEYWORDS: &[&str] = &[
    "止盈止损订单撤单",
    "撤销止盈止损",
    "取消止盈止损",
    "cancel tp/sl",
    "cancel tpsl",
    "撤止盈止损",
];
const CANCEL_TRIGGER_KEYWORDS: &[&str] = &[
    "合约计划委托撤单",
    "撤销计划委托",
    "取消计划委托",
    "cancel trigger",
    "cancel conditional",
    "撤计划委托",
];

const ENTRY_KEYWORDS: [&str; 8] = [
    "入场价",
    "入場價",
    "入场点位",
    "开仓价格",
    "开仓价",
    "Entry",
    "entry",
    "Entry Point",
];
const TP_KEYWORDS: [&str; 7] = [
    "止盈价",
    "止盈",
    "目標價",
    "TP",
    "tp",
    "Take Profit",
    "目标价",
];
const SL_KEYWORDS: [&str; 7] = ["止损价", "止损", "止損", "SL", "sl", "Stop Loss", "防守位"];

const TRIGGER_KEYWORDS: [&str; 4] = ["触发价格", "触发价", "Trigger Price", "trigger price"];
const LIMIT_PRICE_KEYWORDS: [&str; 4] = ["委托价格", "委托价", "Limit Price", "limit price"];

fn extract_price_with_patterns(text: &str, keywords: &[&str]) -> Option<f64> {
    for keyword in keywords {
        let escaped = regex::escape(keyword);
        let patterns = [
            format!(
                r"{kw}[\s:：]*(?:约|around|~)?\s*\$?([0-9]+(?:[.,][0-9]+)?)(?:\s*(?:-|到|~)\s*[0-9]+(?:[.,][0-9]+)?)?(?:\s*(?i:USDT|USD))?",
                kw = escaped
            ),
            format!(
                r"([0-9]+(?:[.,][0-9]+)?)(?:\s*(?i:USDT|USD))?\s*{kw}",
                kw = escaped
            ),
            format!(r"{kw}[^0-9]{{0,6}}([0-9]+(?:[.,][0-9]+)?)", kw = escaped),
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(&pattern) {
                if let Some(caps) = re.captures(text) {
                    let raw = caps.get(1)?.as_str().replace(',', "");
                    if let Ok(value) = raw.parse::<f64>() {
                        if value > 0.0 && value < 1_000_000.0 {
                            return Some(value);
                        }
                    }
                }
            }
        }
    }
    None
}

fn matches_any_keyword(text: &str, keywords: &[&str]) -> bool {
    let text_lower = text.to_lowercase();
    keywords
        .iter()
        .any(|kw| text_lower.contains(&kw.to_lowercase()))
}

fn normalize_action_line(line: &str) -> String {
    let mut trimmed = line.trim();

    // 去除 Markdown 粗体包裹
    trimmed = trimmed.trim_start_matches("**").trim_end_matches("**");

    // 去除项目符号
    trimmed = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c == '•');
    trimmed = trimmed.trim_start();

    // 去除冒号以及后续描述内容, 仅保留标题
    if let Some(colon_pos) = trimmed.find(':') {
        trimmed = &trimmed[..colon_pos];
    }

    // 去除数字序号
    while let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() || matches!(first, '.' | ')' | '、') {
            let byte_len = first.len_utf8();
            trimmed = trimmed[byte_len..].trim_start();
        } else {
            break;
        }
    }

    // 再次清理潜在的 Markdown 粗体标记
    trimmed.replace("**", "").trim().to_string()
}

fn detect_direction(text: &str) -> Option<PositionSide> {
    let lower = text.to_lowercase();
    if text.contains("做多")
        || text.contains("多单")
        || lower.contains("long")
        || lower.contains("bull")
    {
        Some(PositionSide::Long)
    } else if text.contains("做空")
        || text.contains("空单")
        || lower.contains("short")
        || lower.contains("bear")
    {
        Some(PositionSide::Short)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct TradingSignal {
    actions: Vec<ParsedAction>,
}

#[derive(Debug, Clone)]
enum ParsedAction {
    LimitOrder(LimitOrderSignal),
    TriggerOrder(TriggerOrderSignal),
    ClosePosition,
    CancelTPSL,
    CancelTriggerOrder,
    SetTPSL { take_profit: f64, stop_loss: f64 },
}

#[derive(Debug, Clone)]
struct LimitOrderSignal {
    direction: Option<PositionSide>,
    entry_price: Option<f64>,
    take_profit: Option<f64>,
    stop_loss: Option<f64>,
}

#[derive(Debug, Clone)]
struct TriggerOrderSignal {
    direction: Option<PositionSide>,
    trigger_price: Option<f64>,
    limit_price: Option<f64>,
    take_profit: Option<f64>,
    stop_loss: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    fn as_position_str(&self) -> &'static str {
        match self {
            PositionSide::Long => "LONG",
            PositionSide::Short => "SHORT",
        }
    }

    fn order_side(&self) -> &'static str {
        match self {
            PositionSide::Long => "BUY",
            PositionSide::Short => "SELL",
        }
    }
}
