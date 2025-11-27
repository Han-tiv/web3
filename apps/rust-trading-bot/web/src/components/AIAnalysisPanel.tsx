import { type ChangeEvent, useCallback, useMemo, useState } from 'react';
import useSWR from 'swr';
import {
  AlertCircle,
  Brain,
  ChevronDown,
  ChevronUp,
  Filter,
  Loader2,
} from 'lucide-react';
import { api } from '../lib/api';
import type { AiHistoryEntry } from '../types';

const ALL_SYMBOLS = 'ALL';

const formatLocalTime = (value?: string) => {
  if (!value) return '--';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'short',
    timeStyle: 'medium',
  }).format(date);
};

const normalizeConfidence = (value?: number | string): number | null => {
  if (value === undefined || value === null) return null;
  const numeric =
    typeof value === 'string' ? Number.parseFloat(value) : Number(value);
  if (Number.isNaN(numeric)) {
    return null;
  }
  const percent = numeric <= 1 ? numeric * 100 : numeric;
  return Math.min(100, Math.max(0, Number(percent.toFixed(1))));
};

const truncateText = (text: string, limit = 220) => {
  if (text.length <= limit) return text;
  return `${text.slice(0, limit)}...`;
};

const toArray = (value?: string | string[]) => {
  if (!value) return [];
  if (Array.isArray(value)) {
    return value.map((item) => String(item).trim()).filter(Boolean);
  }
  return value
    .split(/[,\|]/)
    .map((item) => item.trim())
    .filter(Boolean);
};

const deriveSignalTags = (entry: AiHistoryEntry) => {
  return toArray(entry.signal_type);
};

const getSignalColor = (label: string) => {
  const normalized = label.toLowerCase();
  if (normalized.includes('long') || normalized.includes('buy')) {
    return 'bg-green-500/15 text-green-400 border border-green-500/30';
  }
  if (normalized.includes('short') || normalized.includes('sell')) {
    return 'bg-red-500/15 text-red-400 border border-red-500/30';
  }
  if (
    normalized.includes('hold') ||
    normalized.includes('wait') ||
    normalized.includes('flat')
  ) {
    return 'bg-yellow-500/15 text-yellow-300 border border-yellow-500/30';
  }
  if (normalized.includes('close') || normalized.includes('exit')) {
    return 'bg-orange-500/15 text-orange-300 border border-orange-500/30';
  }
  return 'bg-gray-800/70 text-gray-300 border border-gray-700';
};

const getAnalysisContent = (entry: AiHistoryEntry) => {
  return entry.reason?.trim() ?? '';
};

const getTimestampValue = (entry: AiHistoryEntry) => entry.timestamp ?? '';

const buildRowKey = (
  entry: AiHistoryEntry,
  index: number,
): string | number => {
  return entry.id ?? `${entry.symbol}-${getTimestampValue(entry)}-${index}`;
};

export function AIAnalysisPanel() {
  const [symbolFilter, setSymbolFilter] = useState<string>(ALL_SYMBOLS);
  const [expandedRows, setExpandedRows] = useState<Set<string | number>>(
    () => new Set(),
  );

  const {
    data: history,
    error,
    isLoading,
  } = useSWR<AiHistoryEntry[]>(
    'ai-analysis-history',
    () => api.getAiHistory(),
    {
      refreshInterval: 60000,
    },
  );

  const normalizedHistory = useMemo(() => {
    if (!history) return [];
    return [...history].sort((a, b) => {
      const aTime = new Date(getTimestampValue(a)).getTime();
      const bTime = new Date(getTimestampValue(b)).getTime();
      return bTime - aTime;
    });
  }, [history]);

  const symbolOptions = useMemo(() => {
    const unique = new Set(
      normalizedHistory
        .map((entry) => entry.symbol?.toUpperCase())
        .filter((symbol): symbol is string => Boolean(symbol)),
    );
    return Array.from(unique).sort();
  }, [normalizedHistory]);

  const filteredHistory = useMemo(() => {
    if (symbolFilter === ALL_SYMBOLS) return normalizedHistory;
    return normalizedHistory.filter(
      (entry) => entry.symbol?.toUpperCase() === symbolFilter,
    );
  }, [normalizedHistory, symbolFilter]);

  const isEmpty = !isLoading && filteredHistory.length === 0;

  const toggleRow = useCallback((rowKey: string | number) => {
    setExpandedRows((prev) => {
      const next = new Set(prev);
      if (next.has(rowKey)) {
        next.delete(rowKey);
      } else {
        next.add(rowKey);
      }
      return next;
    });
  }, []);

  const handleFilterChange = (event: ChangeEvent<HTMLSelectElement>) => {
    setSymbolFilter(event.target.value);
  };

  return (
    <div className="binance-card p-6">
      <div className="flex flex-wrap items-start justify-between gap-4 mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-full bg-blue-500/10 text-blue-300">
            <Brain className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-lg font-semibold binance-text-primary">
              AI 分析面板
            </h3>
            <p className="text-sm binance-text-secondary">
              回溯模型信号、置信度与详细说明
            </p>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-3 text-sm">
          {normalizedHistory.length > 0 && (
            <span className="binance-text-secondary">
              共 {normalizedHistory.length} 条
              {symbolFilter !== ALL_SYMBOLS &&
                `，当前筛选 ${symbolFilter} 共 ${filteredHistory.length} 条`}
            </span>
          )}
          <label className="flex items-center gap-2 px-3 py-1.5 rounded border border-gray-800 bg-gray-900/40">
            <Filter className="w-4 h-4 binance-text-secondary" />
            <select
              className="bg-transparent outline-none text-sm binance-text-primary"
              value={symbolFilter}
              onChange={handleFilterChange}
            >
              <option value={ALL_SYMBOLS}>全部币种</option>
              {symbolOptions.map((symbol) => (
                <option key={symbol} value={symbol}>
                  {symbol}
                </option>
              ))}
            </select>
          </label>
        </div>
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
          AI 分析数据加载中...
        </div>
      )}

      {!error && isEmpty && (
        <div className="text-center py-12 binance-text-secondary">
          <div className="mb-4 flex justify-center opacity-60">
            <Brain className="w-14 h-14" />
          </div>
          <div className="text-lg font-semibold mb-2">暂无匹配的分析记录</div>
          <div className="text-sm">尝试切换币种或等待新的 AI 信号生成。</div>
        </div>
      )}

      {!error && !isEmpty && !isLoading && (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800 text-left binance-text-secondary">
                <th className="py-3 px-4 font-medium">时间</th>
                <th className="py-3 px-4 font-medium">币种</th>
                <th className="py-3 px-4 font-medium">信号类型</th>
                <th className="py-3 px-4 font-medium text-right">置信度</th>
                <th className="py-3 px-4 font-medium">分析内容</th>
              </tr>
            </thead>
            <tbody>
              {filteredHistory.map((entry, index) => {
                const rowKey = buildRowKey(entry, index);
                const tags = deriveSignalTags(entry);
                const confidence = normalizeConfidence(entry.confidence);
                const analysisText = getAnalysisContent(entry);
                const isExpanded = expandedRows.has(rowKey);

                return (
                  <tr
                    key={rowKey}
                    className="border-b border-gray-900/60 hover:bg-gray-900/30 transition-colors align-top"
                  >
                    <td className="py-4 px-4 binance-text-secondary whitespace-nowrap">
                      {formatLocalTime(getTimestampValue(entry))}
                    </td>
                    <td className="py-4 px-4 font-semibold binance-text-primary">
                      {entry.symbol ?? '--'}
                    </td>
                    <td className="py-4 px-4">
                      {tags.length > 0 ? (
                        <div className="flex flex-wrap gap-2">
                          {tags.map((tag) => (
                            <span
                              key={`${rowKey}-${tag}`}
                              className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${getSignalColor(
                                tag,
                              )}`}
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="binance-text-secondary">--</span>
                      )}
                    </td>
                    <td className="py-4 px-4 text-right">
                      {confidence !== null ? (
                        <div>
                          <div className="text-sm font-semibold binance-text-primary">
                            {confidence.toFixed(1)}%
                          </div>
                          <div className="h-2 bg-gray-800 rounded mt-2 overflow-hidden">
                            <div
                              className="h-full rounded bg-gradient-to-r from-emerald-400 to-green-500 transition-all"
                              style={{ width: `${confidence}%` }}
                            />
                          </div>
                        </div>
                      ) : (
                        <span className="binance-text-secondary">--</span>
                      )}
                    </td>
                    <td className="py-4 px-4 binance-text-secondary">
                      {analysisText ? (
                        <div className="space-y-2">
                          <div className="whitespace-pre-line break-words">
                            {isExpanded
                              ? analysisText
                              : truncateText(analysisText)}
                          </div>
                          {analysisText.length > 220 && (
                            <button
                              type="button"
                              onClick={() => toggleRow(rowKey)}
                              className="inline-flex items-center gap-1 text-xs font-medium text-emerald-300 hover:text-emerald-200 transition-colors"
                            >
                              {isExpanded ? '收起' : '展开'}
                              {isExpanded ? (
                                <ChevronUp className="w-3 h-3" />
                              ) : (
                                <ChevronDown className="w-3 h-3" />
                              )}
                            </button>
                          )}
                        </div>
                      ) : (
                        <span>--</span>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
