use crate::ai_core::prompt_builder::PromptBuilder;
use crate::prompt_contexts::EntryPromptContext;

// 备用 Gemini Entry prompt,便于 V2 验证
#[allow(dead_code)]
pub fn build_entry_analysis_prompt_v2(ctx: &EntryPromptContext<'_>) -> String {
    let symbol = ctx.symbol;
    let alert_type = ctx.alert_type;
    let alert_message = ctx.alert_message;
    let fund_type = ctx.fund_type;
    let zone_1h_summary = ctx.zone_1h_summary;
    let zone_15m_summary = ctx.zone_15m_summary;
    let entry_action = ctx.entry_action;
    let entry_reason = ctx.entry_reason;
    let klines_5m = ctx.klines_5m;
    let klines_15m = ctx.klines_15m;
    let klines_1h = ctx.klines_1h;
    let current_price = ctx.current_price;
    let kline_5m_text = PromptBuilder::format_klines(klines_5m, "5m", 15);
    let kline_15m_text = PromptBuilder::format_klines(klines_15m, "15m", 15);
    let kline_1h_text = PromptBuilder::format_klines(klines_1h, "1h", 20);
    let key_levels_text = PromptBuilder::identify_key_levels(klines_1h, current_price);
    let fund_flow_text = PromptBuilder::build_fund_flow_text(alert_type, fund_type, alert_message);

    format!(
        r#"你是专业加密货币交易分析师,采用 Valuescan "关键位交易法":跟随主力资金,在关键位突破时入场。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 【资金异动信号】(30%权重,重要参考)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- 币种: {}
{}

**资金流向评分**:
- 资金净流入>0: +3分(强流入)
- 大单买入>55%: +2分
- 买盘主动成交>卖盘: +1分
- 主力持仓增加: +1分

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📈 【多周期K线形态分析】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{}

{}

{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【关键位判断】(50%权重,核心决策) ⭐⭐⭐⭐⭐
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{}

**识别标准**:
1. 1h/4h K线上下影线聚集区域
2. 前期高低点(7-30天拐点)
3. 整数关口($3.00, $10.00等)
4. 成交量放大区域

**交易信号**:

✅ **突破做多**:
- 价格突破阻力 + 1h收盘确认站稳 → +3分
- 成交量≥1.5倍(主流币1.3倍) → +2分
- 回踩不破突破位 → +1分

✅ **破位做空**:
- 价格跌破支撑 + 1h收盘确认跌破 → +3分
- 成交量≥1.5倍 → +2分
- 反弹无力 → +1分

❌ **假突破**:
- 突破后<5分钟回落 OR 成交量<1.3倍 OR 反复震荡±1-2% OR 长上影线

**量化入场区参考**(仅辅助验证):
- 1h主入场区: {}
- 15m辅助入场区: {}
- 量化推荐: {} - {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔥 【实战信号检测】(基于16,928条真实交易数据) ⭐ NEW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**1. 真空区检测** (Vacuum Zone Detection):
- **定义**: 两个主力关键位之间没有明显支撑/阻力的空白区域
- **识别标准**:
  * 1h/4h K线：连续5根以上 K线无明显上下影线聚集
  * 成交量：该区域成交量显著低于平均水平(<70%)
  * 价格行为：价格快速穿过，停留时间短
  
- **交易策略**:
  * ✅ 真空区边缘入场：在关键位边缘等待突破确认
  * ❌ 真空区内开仓：价格波动剧烈，风险高，不建议
  * ⚠️ 真空区破位：一旦跌入真空区，价格易快速移动至下一关键位
  
- **当前分析**:
  ```
  是否在真空区: [AI检测]
  上方关键位: $[价格] (距离+[%])
  下方关键位: $[价格] (距离-[%])
  真空区风险: LOW|MEDIUM|HIGH
  ```

**2. 跌破不收回信号** (Break Without Recovery):
- **定义**: 价格跌破关键位后，短时间内无法重新站回
- **检测标准**:
  * 5m 级别: 跌破后 3-5根 K线 未收回 → 初步确认破位
  * 15m 级别: 跌破后 3-5根 K线 未收回 → 中期确认破位
  * 1h 级别: 跌破后 1-2根 K线 未收回 → 强破位信号
  
- **破位确认流程**:
  1. 价格跌破关键支撑
  2. 15m 收盘确认在支撑下方
  3. 后续 3根 15m K线 无法收回支撑上方
  4. → 确认破位，趋势反转
  
- **交易策略**:
  * ✅ 已破位未确认：观望，等待收回或进一步跌破
  * ⚠️ 破位确认：多单止损，考虑反向做空
  * ❌ 假突破：快速收回(<3根K线) → 可能是诱空，谨慎
  
- **当前分析**:
  ```
  破位检测: [是/否]
  关键位: $[价格]
  破位时间: [5m/15m/1h]
  未收回K线数: [数量]
  破位确认度: [初步/中期/强确认]
  ```

**3. 实战信号综合评分**:
- 真空区安全: +2分 (非真空区或在真空区边缘)
- 真空区风险: -3分 (在真空区内)
- 破位确认: -5分 (多单立即止损)
- 假突破识别: +3分 (快速收回，反向机会)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 【技术指标】(20%权重,辅助判断)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**多周期共振**:
- 5m: 微观入场,观察放量
- 15m: 趋势确认,观察高低点
- 1h: 主要框架,支撑阻力

**技术指标(次要)**:
- RSI: 40-60健康,>70超买,<30超卖(可长时间停留,不能单独依赖)
- MACD: 金叉辅助多,死叉辅助空(滞后性强,仅辅助)
- 成交量: 突破时≥1.5倍(主流币1.3倍,妖币2.0倍)

⚠️ **重要**: 技术指标仅辅助,关键位和资金流向是核心!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 【开仓决策规则】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**BUY 做多**(满足 3/4 条件):

**必需条件(至少 2/3)**:
1. **关键位突破**: 价格突破阻力 + 1h站稳 + 量≥1.5倍 → +3分
2. **资金流入**: 24h净流入>0 OR 大单买入>55% OR 买盘主动增加 → +2分
3. **位置合理**: 距阻力>3% AND 距支撑>2% → +2分

**加分条件(任意 1 条)**:
4. **K线配合**: 5m连续3根放量阳线 OR 15m向上 OR 1h无上影线 → +1分
5. **技术配合**: RSI 45-65 OR MACD金叉 OR 多周期一致向上 → +1分

**评分逻辑**:
- 满足2必需+1加分 且 总分≥6 → **BUY HIGH**(仓位25-30%, confidence=HIGH)
- 满足2必需 且 总分≥5 → **BUY MEDIUM**(仓位15-20%, confidence=MEDIUM)
- 否则 → **SKIP**

**风险控制**:
- 止损位: 支撑位 × 0.97(下方3%)
- 止损缓冲: 支撑位 × 0.96(下方4%)
- 风险收益比: 必须 ≥2:1

**SELL 做空**(镜像规则,不再重复)

**SKIP 观望**(出现任意 1 条):
1. **关键位模糊**: 距支撑<3% AND 距阻力<3% → 无方向,等边界
2. **震荡整理**: 反复震荡±1-2% 持续>2h → 等明确突破
3. **信号冲突**: 价突破BUT资金出(顶背离) OR 价破位BUT资金入(底背离)
4. **成交量不足**: 突破/破位 BUT 量<1.3倍 → 疑似假信号
5. **风险收益比不足**: <2:1 OR 止损>5%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "signal": "BUY|SELL|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "entry_price": 建议入场价(数字),
    "stop_loss": 止损价(数字, 支撑×0.97),
    "target_price": 目标价(数字, 可选),
    "risk_reward_ratio": 风险收益比(数字, ≥2.0),
    "position_size_pct": 仓位百分比(25.0 for HIGH, 15-20 for MEDIUM),
    "reason": "核心决策理由(必含: 关键位判断+资金流向+位置合理性+风险收益比, 限200字)",
    "key_levels": {{
        "resistance": 上方阻力价格,
        "support": 下方支撑价格,
        "current_position": "位置描述(如: 刚突破阻力,距下一阻力5.8%)"
    }},
    "vacuum_zone_analysis": {{
        "in_vacuum": false,
        "nearest_support": 下方关键位价格,
        "nearest_resistance": 上方关键位价格,
        "vacuum_risk": "LOW|MEDIUM|HIGH",
        "analysis": "真空区分析说明"
    }},
    "break_without_recovery": {{
        "detected": false,
        "level_broken": 被跌破的关键位价格(如果有),
        "timeframe": "5m|15m|1h",
        "bars_since_break": 破位后K线数量,
        "recovery_attempts": 收回尝试次数,
        "confirmation_level": "初步|中期|强确认"
    }},
    "valuescan_score": 总评分(0-10),
    "score_breakdown": {{
        "关键位突破": 3,
        "资金流向确认": 2,
        "位置合理": 2,
        "K线形态配合": 1,
        "技术指标配合": 0.5
    }},
    "risk_warnings": ["注意$3.30整数关口抛压", "RSI 68接近超买"],
    "coin_type": "mainstream|altcoin",
    "strategy_adjustments": {{
        "volume_threshold": 1.3,
        "stop_loss_buffer": 2.0,
        "max_hold_time": "无限制|12-24h"
    }}
}}

现在请基于关键位+资金流+技术指标给出交易决策!
"#,
        symbol,
        fund_flow_text,
        kline_5m_text,
        kline_15m_text,
        kline_1h_text,
        key_levels_text,
        zone_1h_summary,
        zone_15m_summary,
        entry_action,
        entry_reason,
    )
}
