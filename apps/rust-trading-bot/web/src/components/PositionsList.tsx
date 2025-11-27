import useSWR from 'swr';
import { api } from '../lib/api';
import { Wallet, TrendingUp, TrendingDown, RefreshCw } from 'lucide-react';
import { format } from 'date-fns';

export function PositionsList() {
  const { data: positions, error, mutate } = useSWR('positions', api.getPositions, {
    refreshInterval: 5000, // 5秒刷新
  });

  if (error) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-4 binance-text-primary">当前持仓</h3>
        <div className="flex items-center gap-3 p-4 rounded bg-red-900/10 border border-red-900/20">
          <Wallet className="w-6 h-6 binance-red" />
          <div>
            <div className="font-semibold binance-red">加载失败</div>
            <div className="text-sm binance-text-secondary">{error.message}</div>
          </div>
        </div>
      </div>
    );
  }

  if (!positions || positions.length === 0) {
    return (
      <div className="binance-card p-6">
        <h3 className="text-lg font-semibold mb-4 binance-text-primary">当前持仓</h3>
        <div className="text-center py-12 binance-text-secondary">
          <div className="mb-4 flex justify-center opacity-50">
            <Wallet className="w-16 h-16" />
          </div>
          <div className="text-lg font-semibold mb-2">暂无持仓</div>
          <div className="text-sm">机器人发现信号后将自动建仓</div>
        </div>
      </div>
    );
  }

  return (
    <div className="binance-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold binance-text-primary">
          当前持仓 ({positions.length})
        </h3>
        <button
          onClick={() => mutate()}
          className="p-2 rounded hover:bg-gray-800 transition-colors"
          title="刷新"
        >
          <RefreshCw className="w-4 h-4 binance-text-secondary" />
        </button>
      </div>

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
                当前价
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                数量
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                盈亏
              </th>
              <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                杠杆
              </th>
              <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                入场时间
              </th>
            </tr>
          </thead>
          <tbody>
            {positions.map((position) => {
              const isProfitable = position.pnl >= 0;
              const isLong = position.side === 'LONG';

              return (
                <tr
                  key={position.symbol}
                  className="border-b border-gray-800 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="py-3 px-4">
                    <div className="font-semibold binance-text-primary">
                      {position.symbol}
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
                      {position.side}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-right binance-text-primary">
                    ${position.entry_price.toFixed(4)}
                  </td>
                  <td className="py-3 px-4 text-right binance-text-primary">
                    ${position.current_price.toFixed(4)}
                  </td>
                  <td className="py-3 px-4 text-right binance-text-secondary">
                    {position.quantity.toFixed(4)}
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div
                      className={`font-semibold ${
                        isProfitable ? 'binance-green' : 'binance-red'
                      }`}
                    >
                      {isProfitable ? '+' : ''}${position.pnl.toFixed(2)}
                    </div>
                    <div
                      className={`text-sm ${
                        isProfitable ? 'binance-green' : 'binance-red'
                      }`}
                    >
                      {isProfitable ? '+' : ''}{position.pnl_pct.toFixed(2)}%
                    </div>
                  </td>
                  <td className="py-3 px-4 text-right">
                    <span className="binance-gold">{position.leverage}x</span>
                  </td>
                  <td className="py-3 px-4 binance-text-secondary text-sm">
                    {format(new Date(position.entry_time), 'MM-dd HH:mm')}
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
