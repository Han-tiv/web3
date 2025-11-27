import useSWR from 'swr';
import { api } from '../lib/api';
import { History, TrendingUp, TrendingDown } from 'lucide-react';
import { format, formatDuration, intervalToDuration } from 'date-fns';
import { zhCN } from 'date-fns/locale';

export function TradesHistory() {
  const { data: trades, error } = useSWR('trades', () => api.getTrades(50), {
    refreshInterval: 30000, // 30秒刷新
  });

  if (error) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-4 binance-text-primary">交易历史</h3>
        <div className="flex items-center gap-3 p-4 rounded bg-red-900/10 border border-red-900/20">
          <History className="w-6 h-6 binance-red" />
          <div>
            <div className="font-semibold binance-red">加载失败</div>
            <div className="text-sm binance-text-secondary">{error.message}</div>
          </div>
        </div>
      </div>
    );
  }

  if (!trades || trades.length === 0) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-4 binance-text-primary">交易历史</h3>
        <div className="text-center py-12 binance-text-secondary">
          <div className="mb-4 flex justify-center opacity-50">
            <History className="w-16 h-16" />
          </div>
          <div className="text-lg font-semibold mb-2">暂无交易记录</div>
          <div className="text-sm">完成交易后记录将显示在这里</div>
        </div>
      </div>
    );
  }

  const formatHoldTime = (seconds: number) => {
    const duration = intervalToDuration({ start: 0, end: seconds * 1000 });
    return formatDuration(duration, {
      format: ['days', 'hours', 'minutes'],
      locale: zhCN,
      delimiter: ' ',
    });
  };

  return (
    <div className="binance-card p-6">
      <h3 className="text-lg font-semibold mb-4 binance-text-primary">
        交易历史 (最近{trades.length}条)
      </h3>

      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-800">
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                交易对
              </th>
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                方向
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                入场价
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                出场价
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                盈亏
              </th>
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                持仓时长
              </th>
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                开仓时间
              </th>
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                平仓时间
              </th>
            </tr>
          </thead>
          <tbody>
            {trades.map((trade) => {
              const isProfitable = trade.pnl >= 0;
              const isLong = trade.side === 'LONG';

              return (
                <tr
                  key={trade.id}
                  className="border-b border-gray-800 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="py-3 px-4">
                    <div className="font-semibold binance-text-primary">
                      {trade.symbol}
                    </div>
                  </td>
                  <td className="py-3 px-4">
                    <span
                      className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium ${
                        isLong
                          ? 'bg-green-900/20 text-green-500'
                          : 'bg-red-900/20 text-red-500'
                      }`}
                    >
                      {isLong ? (
                        <TrendingUp className="w-3 h-3" />
                      ) : (
                        <TrendingDown className="w-3 h-3" />
                      )}
                      {trade.side}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-right binance-text-primary">
                    ${trade.entry_price.toFixed(4)}
                  </td>
                  <td className="py-3 px-4 text-right binance-text-primary">
                    ${trade.exit_price.toFixed(4)}
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div
                      className={`font-semibold ${
                        isProfitable ? 'binance-green' : 'binance-red'
                      }`}
                    >
                      {isProfitable ? '+' : ''}${trade.pnl.toFixed(2)}
                    </div>
                    <div
                      className={`text-sm ${
                        isProfitable ? 'binance-green' : 'binance-red'
                      }`}
                    >
                      {isProfitable ? '+' : ''}{trade.pnl_pct.toFixed(2)}%
                    </div>
                  </td>
                  <td className="py-3 px-4 binance-text-secondary text-sm">
                    {formatHoldTime(trade.hold_duration)}
                  </td>
                  <td className="py-3 px-4 binance-text-secondary text-sm">
                    {format(new Date(trade.entry_time), 'MM-dd HH:mm:ss')}
                  </td>
                  <td className="py-3 px-4 binance-text-secondary text-sm">
                    {format(new Date(trade.exit_time), 'MM-dd HH:mm:ss')}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
