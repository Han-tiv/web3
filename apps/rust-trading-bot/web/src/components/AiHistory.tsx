import { useEffect, useMemo, useState } from 'react';
import useSWR from 'swr';
import { format } from 'date-fns';
import { AlertCircle, Brain, Loader2, ChevronLeft, ChevronRight } from 'lucide-react';
import { api } from '../lib/api';
import type { AiHistoryEntry } from '../types';

const PAGE_SIZE = 10;

const humanizeLabel = (value?: string) => {
  if (!value) return '--';
  return value
    .split(/[_\s]+/)
    .filter(Boolean)
    .map((chunk) => chunk.charAt(0).toUpperCase() + chunk.slice(1).toLowerCase())
    .join(' ');
};

const formatTimestamp = (timestamp?: string) => {
  if (!timestamp) return '--';
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    return timestamp;
  }
  return format(date, 'MM-dd HH:mm:ss');
};

const formatConfidence = (value?: number) => {
  if (value === undefined || value === null || Number.isNaN(value)) {
    return '--';
  }
  // 兼容 0-1 与 0-100 两种置信度返回形式
  const normalized = value <= 1 ? value * 100 : value;
  return `${normalized.toFixed(1)}%`;
};

const getDecisionBadge = (decision?: string) => {
  if (!decision) {
    return { label: '--', className: 'bg-gray-800/60 text-gray-300' };
  }
  const normalized = decision.toLowerCase();
  if (normalized.includes('long') || normalized.includes('buy')) {
    return { label: humanizeLabel(decision), className: 'bg-green-900/20 text-green-400' };
  }
  if (normalized.includes('short') || normalized.includes('sell')) {
    return { label: humanizeLabel(decision), className: 'bg-red-900/20 text-red-400' };
  }
  if (normalized.includes('close') || normalized.includes('flat')) {
    return { label: humanizeLabel(decision), className: 'bg-yellow-900/20 text-yellow-400' };
  }
  return { label: humanizeLabel(decision), className: 'bg-gray-800/60 text-gray-300' };
};

export function AiHistory() {
  const [page, setPage] = useState(1);
  const {
    data: history,
    error,
    isLoading,
  } = useSWR<AiHistoryEntry[]>('ai-history', () => api.getAiHistory(), {
    refreshInterval: 60000,
  });

  useEffect(() => {
    setPage(1);
  }, [history?.length]);

  const normalizedHistory = useMemo(() => {
    if (!history) return [];
    // 依据时间倒序排列，保证最新记录优先展示
    return [...history].sort((a, b) => {
      const aTime = new Date(a.timestamp ?? 0).getTime();
      const bTime = new Date(b.timestamp ?? 0).getTime();
      return bTime - aTime;
    });
  }, [history]);

  const totalPages = Math.max(1, Math.ceil(normalizedHistory.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);
  const start = (currentPage - 1) * PAGE_SIZE;
  const paginatedHistory = normalizedHistory.slice(start, start + PAGE_SIZE);
  const isEmpty = !isLoading && normalizedHistory.length === 0;

  const handlePrev = () => {
    setPage((prev) => Math.max(1, prev - 1));
  };

  const handleNext = () => {
    setPage((prev) => Math.min(totalPages, prev + 1));
  };

  return (
    <div className="binance-card p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-full bg-green-500/10 text-green-400">
            <Brain className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-lg font-semibold binance-text-primary">AI 分析历史</h3>
            <p className="text-sm binance-text-secondary">追踪模型结论与决策依据</p>
          </div>
        </div>
        {normalizedHistory.length > 0 && (
          <div className="text-sm binance-text-secondary">共 {normalizedHistory.length} 条记录</div>
        )}
      </div>

      {error && (
        <div className="flex items-center gap-3 p-4 rounded bg-red-900/10 border border-red-900/20">
          <AlertCircle className="w-5 h-5 binance-red" />
          <div>
            <div className="font-semibold binance-red">加载失败</div>
            <div className="text-sm binance-text-secondary">{error.message}</div>
          </div>
        </div>
      )}

      {!error && isLoading && (
        <div className="flex items-center justify-center py-12 binance-text-secondary">
          <Loader2 className="w-5 h-5 mr-3 animate-spin" />
          正在获取 AI 历史...
        </div>
      )}

      {!error && isEmpty && (
        <div className="text-center py-12 binance-text-secondary">
          <div className="mb-4 flex justify-center opacity-60">
            <Brain className="w-14 h-14" />
          </div>
          <div className="text-lg font-semibold mb-2">暂无 AI 分析记录</div>
          <div className="text-sm">当 AI 输出交易建议后，详细原因将显示在此。</div>
        </div>
      )}

      {!error && !isEmpty && !isLoading && (
        <>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-800">
                  <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                    时间
                  </th>
                  <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                    币种
                  </th>
                  <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                    分析类型
                  </th>
                  <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                    决策
                  </th>
                  <th className="text-right py-3 px-4 binance-text-secondary text-sm font-medium">
                    置信度
                  </th>
                  <th className="text-left py-3 px-4 binance-text-secondary text-sm font-medium">
                    原因
                  </th>
                </tr>
              </thead>
              <tbody>
                {paginatedHistory.map((entry, index) => {
                  const badge = getDecisionBadge(entry.decision);
                  return (
                    <tr
                      key={
                        entry.id ??
                        `${entry.timestamp}-${entry.symbol}-${index + start}`
                      }
                      className="border-b border-gray-800 hover:bg-gray-800/30 transition-colors"
                    >
                      <td className="py-3 px-4 binance-text-secondary text-sm">
                        {formatTimestamp(entry.timestamp)}
                      </td>
                      <td className="py-3 px-4">
                        <div className="font-semibold binance-text-primary">
                          {entry.symbol ?? '--'}
                        </div>
                      </td>
                      <td className="py-3 px-4">
                        <span className="inline-flex items-center px-2 py-1 rounded bg-gray-800/60 text-xs font-medium binance-text-secondary">
                          {humanizeLabel(entry.signal_type)}
                        </span>
                      </td>
                      <td className="py-3 px-4">
                        <span
                          className={`inline-flex items-center px-2 py-1 rounded text-xs font-semibold ${badge.className}`}
                        >
                          {badge.label}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-right binance-text-primary">
                        {formatConfidence(entry.confidence)}
                      </td>
                      <td className="py-3 px-4 binance-text-secondary text-sm whitespace-pre-line break-words">
                        {entry.reason ?? '--'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          <div className="flex items-center justify-between mt-4 text-sm binance-text-secondary">
            <span>
              第 {currentPage}/{totalPages} 页 · 每页 {PAGE_SIZE} 条
            </span>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={handlePrev}
                disabled={currentPage === 1}
                className="inline-flex items-center gap-1 px-3 py-1.5 rounded border border-gray-800 disabled:opacity-40 disabled:cursor-not-allowed hover:border-gray-600 transition-colors"
              >
                <ChevronLeft className="w-4 h-4" />
                上一页
              </button>
              <button
                type="button"
                onClick={handleNext}
                disabled={currentPage === totalPages}
                className="inline-flex items-center gap-1 px-3 py-1.5 rounded border border-gray-800 disabled:opacity-40 disabled:cursor-not-allowed hover:border-gray-600 transition-colors"
              >
                下一页
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
