import type {
  AccountSummary,
  EquityPoint,
  Position,
  TradeRecord,
  SystemStatus,
  AiHistoryEntry,
} from '../types';

const API_BASE = '/api';

async function fetchJSON<T>(endpoint: string): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`);
  if (!response.ok) {
    throw new Error(`API请求失败: ${response.statusText}`);
  }
  return response.json();
}

export const api = {
  // 获取系统运行状态
  getStatus: async (): Promise<SystemStatus> => {
    return fetchJSON<SystemStatus>('/status');
  },

  // 获取 AI 决策历史
  getAiHistory: async (): Promise<AiHistoryEntry[]> => {
    return fetchJSON<AiHistoryEntry[]>('/ai-history');
  },

  // 获取账户摘要
  getAccount: async (): Promise<AccountSummary> => {
    return fetchJSON<AccountSummary>('/account');
  },

  // 获取权益历史曲线
  getEquityHistory: async (): Promise<EquityPoint[]> => {
    return fetchJSON<EquityPoint[]>('/equity-history');
  },

  // 获取当前持仓
  getPositions: async (): Promise<Position[]> => {
    return fetchJSON<Position[]>('/positions');
  },

  // 获取交易历史
  getTrades: async (limit: number = 50): Promise<TradeRecord[]> => {
    return fetchJSON<TradeRecord[]>(`/trades?limit=${limit}`);
  },

  // 手动平仓
  closePosition: async (symbol: string): Promise<void> => {
    const response = await fetch(`${API_BASE}/positions/${symbol}/close`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`平仓失败: ${response.statusText}`);
    }
  },
};
