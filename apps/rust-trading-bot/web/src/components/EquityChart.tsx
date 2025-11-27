import { useCallback, useState } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import useSWR from 'swr';
import { api } from '../lib/api';
import {
  BarChart3,
  DollarSign,
  Percent,
  TrendingUp,
  TrendingDown,
  Loader2,
  RefreshCw,
} from 'lucide-react';
import { format } from 'date-fns';
import type { EquityPoint } from '../types';

export function EquityChart() {
  const [displayMode, setDisplayMode] = useState<'dollar' | 'percent'>('dollar');

  const {
    data: history,
    error: historyError,
    isLoading: isHistoryLoading,
    isValidating: isHistoryValidating,
    mutate: revalidateHistory,
  } = useSWR<EquityPoint[]>('equity-history', api.getEquityHistory, {
    refreshInterval: 30000,
    dedupingInterval: 15000,
    revalidateOnFocus: false,
  });

  const isInitialLoading = isHistoryLoading;
  const isRefreshing = isHistoryValidating && !isHistoryLoading;

  const { data: account } = useSWR('account', api.getAccount, {
    refreshInterval: 15000, // 15秒刷新
  });

  // ✅ 将 useCallback 移到所有条件判断之前（React Hooks 规则）
  const handleManualRefresh = useCallback(() => {
    void revalidateHistory();
  }, [revalidateHistory]);

  if (historyError) {
    return (
      <div className="binance-card p-6">
        <div className="flex items-center gap-3 p-4 rounded bg-red-900/10 border border-red-900/20">
          <BarChart3 className="w-6 h-6 binance-red" />
          <div>
            <div className="font-semibold binance-red">加载失败</div>
            <div className="text-sm binance-text-secondary">{historyError.message}</div>
          </div>
        </div>
      </div>
    );
  }

  if (isInitialLoading) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-6 binance-text-primary">账户权益曲线</h3>
        <div className="flex flex-col items-center justify-center py-16 gap-3 binance-text-secondary">
          <Loader2 className="w-8 h-8 animate-spin text-yellow-500" />
          <div className="text-sm">权益曲线加载中...</div>
        </div>
      </div>
    );
  }

  const validHistory =
    (history ?? []).filter(
      (point) => Number.isFinite(point.total_equity) && point.total_equity > 0,
    ); // 只过滤掉明显异常的权益点

  if (!validHistory || validHistory.length === 0) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-6 binance-text-primary">账户权益曲线</h3>
        <div className="text-center py-16 binance-text-secondary">
          <div className="mb-4 flex justify-center opacity-50">
            <BarChart3 className="w-16 h-16" />
          </div>
          <div className="text-lg font-semibold mb-2">暂无历史数据</div>
          <div className="text-sm">机器人运行后数据将出现在这里</div>
        </div>
      </div>
    );
  }

  const chartData = validHistory.map((point) => ({
    time: format(new Date(point.timestamp), 'MM-dd HH:mm'),
    value: displayMode === 'dollar' ? point.total_equity : point.pnl_pct,
    pnl: point.pnl,
    pnl_pct: point.pnl_pct,
  }));

  const latestPoint = validHistory[validHistory.length - 1];
  const isProfitable = latestPoint.pnl >= 0;
  const initialBalance = account?.initial_balance || 0;

  return (
    <div className="binance-card p-6">
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between mb-6">
        <h3 className="text-lg font-semibold binance-text-primary">账户权益曲线</h3>
        <div className="flex flex-wrap gap-2">
          <button
            onClick={() => setDisplayMode('dollar')}
            className={`px-4 py-2 rounded text-sm font-medium transition-colors ${
              displayMode === 'dollar'
                ? 'bg-yellow-500/20 text-yellow-500 border border-yellow-500/50'
                : 'bg-gray-800 binance-text-secondary border border-gray-700 hover:border-gray-600'
            }`}
          >
            <DollarSign className="w-4 h-4 inline mr-1" />
            美元
          </button>
          <button
            onClick={() => setDisplayMode('percent')}
            className={`px-4 py-2 rounded text-sm font-medium transition-colors ${
              displayMode === 'percent'
                ? 'bg-yellow-500/20 text-yellow-500 border border-yellow-500/50'
                : 'bg-gray-800 binance-text-secondary border border-gray-700 hover:border-gray-600'
            }`}
          >
            <Percent className="w-4 h-4 inline mr-1" />
            百分比
          </button>
          <button
            onClick={handleManualRefresh}
            disabled={isRefreshing}
            className={`px-4 py-2 rounded text-sm font-medium border transition-colors flex items-center gap-1 ${
              isRefreshing
                ? 'border-gray-600 bg-gray-800 binance-text-secondary cursor-not-allowed opacity-70'
                : 'border-gray-700 bg-gray-800 binance-text-secondary hover:border-gray-500'
            }`}
          >
            <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
            {isRefreshing ? '刷新中' : '刷新'}
          </button>
        </div>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="bg-gray-800/50 p-4 rounded">
          <div className="text-sm binance-text-secondary mb-1">当前权益</div>
          <div className="text-xl font-bold binance-text-primary">
            ${latestPoint.total_equity.toFixed(2)}
          </div>
        </div>
        <div className="bg-gray-800/50 p-4 rounded">
          <div className="text-sm binance-text-secondary mb-1">盈亏金额</div>
          <div className={`text-xl font-bold flex items-center gap-1 ${isProfitable ? 'binance-green' : 'binance-red'}`}>
            {isProfitable ? <TrendingUp className="w-5 h-5" /> : <TrendingDown className="w-5 h-5" />}
            ${Math.abs(latestPoint.pnl).toFixed(2)}
          </div>
        </div>
        <div className="bg-gray-800/50 p-4 rounded">
          <div className="text-sm binance-text-secondary mb-1">盈亏百分比</div>
          <div className={`text-xl font-bold ${isProfitable ? 'binance-green' : 'binance-red'}`}>
            {isProfitable ? '+' : ''}{latestPoint.pnl_pct.toFixed(2)}%
          </div>
        </div>
        <div className="bg-gray-800/50 p-4 rounded">
          <div className="text-sm binance-text-secondary mb-1">初始余额</div>
          <div className="text-xl font-bold binance-text-primary">
            ${initialBalance.toFixed(2)}
          </div>
        </div>
      </div>

      {/* 图表 */}
      <div className="h-80">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#2B3139" />
            <XAxis
              dataKey="time"
              stroke="#848E9C"
              style={{ fontSize: '12px' }}
              tick={{ fill: '#848E9C' }}
            />
            <YAxis
              stroke="#848E9C"
              style={{ fontSize: '12px' }}
              tick={{ fill: '#848E9C' }}
              domain={displayMode === 'dollar' ? ['auto', 'auto'] : ['auto', 'auto']}
            />
            <Tooltip
              contentStyle={{
                background: '#1E2329',
                border: '1px solid #2B3139',
                borderRadius: '4px',
                color: '#EAECEF',
              }}
              formatter={(value: number) =>
                displayMode === 'dollar' ? `$${value.toFixed(2)}` : `${value.toFixed(2)}%`
              }
            />
            {displayMode === 'dollar' && initialBalance > 0 && (
              <ReferenceLine
                y={initialBalance}
                stroke="#F0B90B"
                strokeDasharray="3 3"
                label={{
                  value: '初始余额',
                  fill: '#F0B90B',
                  fontSize: 12,
                  position: 'right',
                }}
              />
            )}
            {displayMode === 'percent' && (
              <ReferenceLine
                y={0}
                stroke="#F0B90B"
                strokeDasharray="3 3"
                label={{
                  value: '盈亏平衡线',
                  fill: '#F0B90B',
                  fontSize: 12,
                  position: 'right',
                }}
              />
            )}
            <Line
              type="monotone"
              dataKey="value"
              stroke={isProfitable ? '#0ECB81' : '#F6465D'}
              strokeWidth={2}
              dot={false}
              animationDuration={300}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
