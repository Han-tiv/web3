import useSWR from 'swr';
import { TelegramSignal } from '../types';

const fetcher = (url: string) => fetch(url).then((res) => res.json());

const getSignalIcon = (action: string, processed: boolean) => {
  if (processed) return '✅';
  if (action === 'LONG' || action === 'BUY') return '🚀';
  return '📡';
};

const getActionBgColor = (action: string) => {
  if (action === 'LONG' || action === 'BUY') return 'bg-green-900/20 text-green-500';
  if (action === 'SELL' || action === 'CLOSE/AVOID') return 'bg-red-900/20 text-red-500';
  if (action === 'AVOID') return 'bg-orange-900/20 text-orange-400';
  return 'bg-gray-800 binance-text-secondary';
};

const formatTimestamp = (value: string) => new Date(value).toLocaleString('zh-CN');

const formatOptionalTimestamp = (value?: string | null) => {
  if (!value) return '尚未处理';
  return new Date(value).toLocaleString('zh-CN');
};

const getMessagePreview = (message: string) => {
  if (message.length <= 160) {
    return message;
  }
  return `${message.slice(0, 160)}…`;
};

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

  return (
    <div className="binance-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold binance-text-primary">📡 Telegram 市场信号</h2>
        <span className="text-sm binance-text-secondary">最近 {signals.length} 条信号</span>
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
                  <span className="text-2xl">
                    {getSignalIcon(signal.recommend_action, signal.processed)}
                  </span>
                  <span className="font-bold text-lg binance-text-primary">{signal.symbol}</span>
                </div>

                <div className="space-y-2 text-sm">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="binance-text-secondary">建议:</span>
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${getActionBgColor(
                        signal.recommend_action
                      )}`}
                    >
                      {signal.recommend_action}
                    </span>
                    <span className="text-xs">
                      {signal.processed ? (
                        <span className="text-green-400">已处理</span>
                      ) : (
                        <span className="text-yellow-400">待处理</span>
                      )}
                    </span>
                  </div>

                  <div className="binance-text-primary leading-relaxed">
                    {getMessagePreview(signal.raw_message)}
                  </div>

                  <div className="text-xs binance-text-secondary space-y-1">
                    <div>收到时间: {formatTimestamp(signal.timestamp)}</div>
                    <div>处理时间: {formatOptionalTimestamp(signal.processed_at)}</div>
                    <div>创建时间: {formatTimestamp(signal.created_at)}</div>
                  </div>
                </div>
              </div>
            </div>

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

      <div className="mt-6 p-4 bg-blue-900/10 border border-blue-900/20 rounded-lg">
        <h3 className="text-sm font-semibold text-blue-400 mb-2">信号说明</h3>
        <ul className="text-xs text-blue-300 space-y-1">
          <li>• 当前系统仅保存 Python 端透传的做多信号，recommend_action 固定为 LONG</li>
          <li>• processed 字段用于标记信号是否被交易线程消费</li>
          <li>• 可展开查看完整原始消息，方便人工复核</li>
        </ul>
      </div>
    </div>
  );
}
