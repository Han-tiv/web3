import { useEffect, useState } from 'react';
import useSWR from 'swr';
import { TelegramSignal } from '../types';

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export function TelegramSignals() {
  const { data: signals, error, isLoading } = useSWR<TelegramSignal[]>(
    'http://localhost:8080/api/telegram-signals',
    fetcher,
    {
      refreshInterval: 10000, // 每10秒刷新
    }
  );

  if (isLoading) {
    return (
      <div className="binance-card p-6">
        <h2 className="text-xl font-bold mb-4 binance-text-primary">📡 Telegram 市场信号</h2>
        <p className="binance-text-secondary">加载中...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="binance-card p-6">
        <h2 className="text-xl font-bold mb-4 binance-text-primary">📡 Telegram 市场信号</h2>
        <div className="flex items-center gap-3 p-4 rounded bg-red-900/10 border border-red-900/20">
          <div>
            <div className="font-semibold binance-red">加载失败</div>
            <div className="text-sm binance-text-secondary">{error.message}</div>
          </div>
        </div>
      </div>
    );
  }

  if (!signals || signals.length === 0) {
    return (
      <div className="binance-card p-6">
        <h2 className="text-xl font-bold mb-4 binance-text-primary">📡 Telegram 市场信号</h2>
        <div className="text-center py-12 binance-text-secondary">
          <div className="mb-4 text-4xl">📡</div>
          <div className="text-lg font-semibold mb-2">暂无信号数据</div>
          <div className="text-sm">等待Telegram频道新消息...</div>
        </div>
      </div>
    );
  }

  // 根据评分决定图标
  const getSignalIcon = (score: number) => {
    if (score >= 5) return '🔥🔥';
    if (score >= 3) return '📈';
    if (score >= 1) return '➡️';
    if (score >= -2) return '📉';
    if (score >= -4) return '📉';
    return '🚨';
  };

  // 根据评分决定颜色 - 使用Binance主题
  const getScoreColor = (score: number) => {
    if (score >= 5) return 'binance-green font-bold';
    if (score >= 3) return 'binance-green';
    if (score >= 1) return 'text-blue-400';
    if (score >= -2) return 'text-yellow-400';
    if (score >= -4) return 'text-orange-400';
    return 'binance-red font-bold';
  };

  // 根据建议决定背景色 - 适配深色主题
  const getActionBgColor = (action: string) => {
    if (action === 'BUY') return 'bg-green-900/20 text-green-500';
    if (action === 'SELL' || action === 'CLOSE/AVOID') return 'bg-red-900/20 text-red-500';
    if (action === 'AVOID') return 'bg-orange-900/20 text-orange-400';
    return 'bg-gray-800 binance-text-secondary';
  };

  return (
    <div className="binance-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold binance-text-primary">📡 Telegram 市场信号</h2>
        <span className="text-sm binance-text-secondary">
          最近 {signals.length} 条信号
        </span>
      </div>

      <div className="space-y-4">
        {signals.map((signal) => (
          <div
            key={signal.id}
            className="border border-gray-800 rounded-lg p-4 hover:bg-gray-800/30 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-2xl">{getSignalIcon(signal.score)}</span>
                  <span className="font-bold text-lg binance-text-primary">{signal.symbol}</span>
                  <span className="text-sm binance-text-secondary">{signal.signal_type}</span>
                  <span className={`text-lg font-bold ${getScoreColor(signal.score)}`}>
                    {signal.score > 0 ? '+' : ''}{signal.score}
                  </span>
                </div>

                <div className="space-y-1 text-sm">
                  <div className="flex items-center gap-2">
                    <span className="binance-text-secondary">建议:</span>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${getActionBgColor(
                        signal.recommend_action
                      )}`}
                    >
                      {signal.recommend_action}
                    </span>
                  </div>

                  <div className="flex items-center gap-2">
                    <span className="binance-text-secondary">理由:</span>
                    <span className="binance-text-primary">{signal.reason}</span>
                  </div>

                  <div className="flex items-start gap-2">
                    <span className="binance-text-secondary">关键词:</span>
                    <div className="flex flex-wrap gap-1">
                      {signal.keywords.split(', ').map((keyword, idx) => {
                        const isPositive = keyword.startsWith('+');
                        return (
                          <span
                            key={idx}
                            className={`px-2 py-0.5 rounded text-xs ${
                              isPositive
                                ? 'bg-green-900/20 text-green-500'
                                : 'bg-red-900/20 text-red-500'
                            }`}
                          >
                            {keyword}
                          </span>
                        );
                      })}
                    </div>
                  </div>

                  <div className="text-xs binance-text-secondary mt-2">
                    {new Date(signal.timestamp).toLocaleString('zh-CN')}
                  </div>
                </div>
              </div>
            </div>

            {/* 可展开的原始消息 */}
            <details className="mt-3">
              <summary className="text-xs binance-text-secondary cursor-pointer hover:text-gray-400">
                查看原始消息
              </summary>
              <div className="mt-2 p-2 bg-gray-800/50 rounded text-xs binance-text-secondary whitespace-pre-wrap border border-gray-700">
                {signal.raw_message}
              </div>
            </details>
          </div>
        ))}
      </div>

      {/* 信号解读说明 */}
      <div className="mt-6 p-4 bg-blue-900/10 border border-blue-900/20 rounded-lg">
        <h3 className="text-sm font-semibold text-blue-400 mb-2">信号解读说明</h3>
        <ul className="text-xs text-blue-300 space-y-1">
          <li>• 评分 ≥5: 强烈看多，可考虑入场</li>
          <li>• 评分 3-4: 看多，适度参与</li>
          <li>• 评分 1-2: 中性偏多，观察为主</li>
          <li>• 评分 -2~0: 中性或偏空，谨慎</li>
          <li>• 评分 ≤-3: 看空或风险警告，规避或平仓</li>
        </ul>
      </div>
    </div>
  );
}
