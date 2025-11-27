// Mock API数据 - 用于测试,实际会由Rust后端提供
import type { AccountSummary, EquityPoint, Position, TradeRecord } from '../types';

// 生成模拟的权益历史数据
function generateMockEquityHistory(): EquityPoint[] {
  const points: EquityPoint[] = [];
  const initialBalance = 1000;
  let currentEquity = initialBalance;
  const now = Date.now();

  for (let i = 100; i >= 0; i--) {
    // 模拟随机波动
    const change = (Math.random() - 0.45) * 20;
    currentEquity += change;

    const timestamp = new Date(now - i * 30 * 60 * 1000).toISOString(); // 每30分钟一个点
    const pnl = currentEquity - initialBalance;
    const pnl_pct = (pnl / initialBalance) * 100;

    points.push({
      timestamp,
      total_equity: currentEquity,
      pnl,
      pnl_pct,
    });
  }

  return points;
}

// 模拟当前持仓
const mockPositions: Position[] = [
  {
    symbol: 'BTCUSDT',
    side: 'LONG',
    entry_price: 43250.5,
    current_price: 43580.2,
    quantity: 0.023,
    pnl: 7.58,
    pnl_pct: 0.76,
    entry_time: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    leverage: 5,
  },
  {
    symbol: 'ETHUSDT',
    side: 'SHORT',
    entry_price: 2280.3,
    current_price: 2295.8,
    quantity: 0.5,
    pnl: -7.75,
    pnl_pct: -0.68,
    entry_time: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString(),
    leverage: 3,
  },
];

// 模拟交易历史
const mockTrades: TradeRecord[] = [
  {
    id: '1',
    symbol: 'SOLUSDT',
    side: 'LONG',
    entry_price: 95.2,
    exit_price: 98.5,
    quantity: 10,
    pnl: 33.0,
    pnl_pct: 3.47,
    entry_time: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
    exit_time: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString(),
    hold_duration: 7200,
  },
  {
    id: '2',
    symbol: 'BNBUSDT',
    side: 'SHORT',
    entry_price: 312.5,
    exit_price: 305.2,
    quantity: 3,
    pnl: 21.9,
    pnl_pct: 2.34,
    entry_time: new Date(Date.now() - 10 * 60 * 60 * 1000).toISOString(),
    exit_time: new Date(Date.now() - 8 * 60 * 60 * 1000).toISOString(),
    hold_duration: 7200,
  },
  {
    id: '3',
    symbol: 'ADAUSDT',
    side: 'LONG',
    entry_price: 0.485,
    exit_price: 0.473,
    quantity: 1000,
    pnl: -12.0,
    pnl_pct: -2.47,
    entry_time: new Date(Date.now() - 15 * 60 * 60 * 1000).toISOString(),
    exit_time: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
    hold_duration: 10800,
  },
];

const equityHistory = generateMockEquityHistory();
const latestEquity = equityHistory[equityHistory.length - 1];

const mockAccount: AccountSummary = {
  total_equity: latestEquity.total_equity,
  available_balance: latestEquity.total_equity - 100,
  unrealized_pnl: latestEquity.pnl,
  initial_balance: 1000,
  total_trades: 15,
  win_rate: 0.67,
};

// 导出mock API(仅开发时使用)
export const mockApi = {
  getAccount: async (): Promise<AccountSummary> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return mockAccount;
  },

  getEquityHistory: async (): Promise<EquityPoint[]> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return equityHistory;
  },

  getPositions: async (): Promise<Position[]> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return mockPositions;
  },

  getTrades: async (limit: number = 50): Promise<TradeRecord[]> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return mockTrades.slice(0, limit);
  },

  closePosition: async (symbol: string): Promise<void> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    console.log(`Closing position: ${symbol}`);
  },
};
