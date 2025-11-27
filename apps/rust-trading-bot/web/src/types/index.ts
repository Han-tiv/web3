// 账户摘要
export interface AccountSummary {
  total_equity: number;
  available_balance: number;
  unrealized_pnl: number;
  initial_balance: number;
  total_trades: number;
  win_rate: number;
}

// 权益历史数据点
export interface EquityPoint {
  timestamp: string;
  total_equity: number;
  pnl: number;
  pnl_pct: number;
}

// 持仓信息
export interface Position {
  symbol: string;
  side: string; // "LONG" | "SHORT"
  entry_price: number;
  current_price: number;
  quantity: number;
  pnl: number;
  pnl_pct: number;
  entry_time: string;
  leverage: number;
}

// 交易记录
export interface TradeRecord {
  id: string;
  symbol: string;
  side: string;
  entry_price: number;
  exit_price: number;
  quantity: number;
  pnl: number;
  pnl_pct: number;
  entry_time: string;
  exit_time: string;
  hold_duration: number; // 持仓时长(秒)
}

// 系统运行状态（字段由后端定义，这里使用宽松结构保证兼容）
export interface SystemStatus {
  [key: string]: unknown;
}

// AI 决策或对话历史记录（字段同样由后端返回决定）
export interface AiHistoryEntry {
  id: number;
  timestamp: string;
  symbol: string;
  decision: string;
  confidence: number;
  signal_type: string;
  reason: string;
}

// Telegram 信号记录
export interface TelegramSignal {
  id: number;
  symbol: string;
  signal_type: string; // "强烈看多", "看多", "中性", "看空", "强烈看空"
  score: number; // -21 到 +10
  keywords: string; // 逗号分隔的关键词
  recommend_action: string; // "BUY", "SELL", "WATCH", "AVOID", "CLOSE/AVOID"
  reason: string;
  raw_message: string;
  timestamp: string;
  created_at: string;
}
