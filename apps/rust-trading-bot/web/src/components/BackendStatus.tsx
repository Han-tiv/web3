import useSWR from 'swr';
import { api } from '../lib/api';
import type { SystemStatus } from '../types';

const STATUS_REFRESH_INTERVAL = 10_000;

const STATUS_FIELD_KEYS = {
  online: ['is_running', 'running', 'online', 'status', 'state'],
  uptimeSeconds: ['uptime_seconds', 'uptime', 'runtime_seconds', 'run_time_seconds'],
  uptimeMinutes: ['runtime_minutes', 'uptime_minutes'],
  uptimeHours: ['runtime_hours', 'uptime_hours'],
  uptimeFormatted: ['runtime_formatted', 'uptime_text'],
  lastUpdate: ['last_update', 'last_updated', 'last_update_time', 'last_updated_at', 'updated_at'],
  positionCount: [
    'position_count',
    'positions_count',
    'open_position_count',
    'open_positions',
    'positions',
  ],
  tradeCount: ['trade_count', 'trades_count', 'trades_total', 'total_trades', 'trades'],
  aiAnalysisCount: [
    'ai_analysis_count',
    'ai_decision_count',
    'ai_signal_count',
    'ai_analysis_total',
    'ai_decisions',
  ],
} as const;

const pickFieldValue = (status: SystemStatus | undefined, keys: readonly string[]): unknown => {
  if (!status) {
    return undefined;
  }
  for (const key of keys) {
    const value = status[key];
    if (value !== undefined && value !== null) {
      return value;
    }
  }
  return undefined;
};

const parseBooleanValue = (value: unknown): boolean | undefined => {
  if (typeof value === 'boolean') {
    return value;
  }
  if (typeof value === 'number') {
    if (value === 1) return true;
    if (value === 0) return false;
  }
  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase();
    if (['online', 'running', 'up', 'ok', 'healthy', 'ready', 'true', '1'].includes(normalized)) {
      return true;
    }
    if (['offline', 'down', 'stopped', 'error', 'false', '0'].includes(normalized)) {
      return false;
    }
  }
  return undefined;
};

const parseNumericValue = (value: unknown): number | undefined => {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === 'string') {
    const sanitized = value.replace(/,/g, '').trim();
    if (!sanitized) {
      return undefined;
    }
    const numeric = Number(sanitized);
    if (Number.isFinite(numeric)) {
      return numeric;
    }
    const parsed = parseFloat(sanitized);
    return Number.isFinite(parsed) ? parsed : undefined;
  }
  return undefined;
};

const parseCountValue = (value: unknown): number | undefined => {
  if (Array.isArray(value)) {
    return value.length;
  }
  return parseNumericValue(value);
};

const parseTimestampValue = (value: unknown): Date | undefined => {
  if (!value) {
    return undefined;
  }
  if (value instanceof Date && !Number.isNaN(value.getTime())) {
    return value;
  }
  if (typeof value === 'number' && Number.isFinite(value)) {
    if (value > 1e12) {
      return new Date(value);
    }
    if (value > 1e9) {
      return new Date(value * 1000);
    }
    return new Date(value);
  }
  if (typeof value === 'string') {
    const numeric = Number(value);
    if (Number.isFinite(numeric)) {
      if (numeric > 1e12) {
        return new Date(numeric);
      }
      if (numeric > 1e9) {
        return new Date(numeric * 1000);
      }
      return new Date(numeric);
    }
    const parsed = new Date(value);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed;
    }
  }
  return undefined;
};

const extractRuntimeSeconds = (status: SystemStatus | undefined): number | undefined => {
  const secondsValue = parseNumericValue(pickFieldValue(status, STATUS_FIELD_KEYS.uptimeSeconds));
  if (secondsValue !== undefined) {
    return secondsValue;
  }
  const minutesValue = parseNumericValue(pickFieldValue(status, STATUS_FIELD_KEYS.uptimeMinutes));
  if (minutesValue !== undefined) {
    return minutesValue * 60;
  }
  const hoursValue = parseNumericValue(pickFieldValue(status, STATUS_FIELD_KEYS.uptimeHours));
  if (hoursValue !== undefined) {
    return hoursValue * 3600;
  }
  const formatted = pickFieldValue(status, STATUS_FIELD_KEYS.uptimeFormatted);
  if (typeof formatted === 'string' && formatted.includes(':')) {
    const pieces = formatted.split(':').map((segment) => Number(segment));
    if (pieces.every((segment) => Number.isFinite(segment))) {
      if (pieces.length === 3) {
        return pieces[0] * 3600 + pieces[1] * 60 + pieces[2];
      }
      if (pieces.length === 2) {
        return pieces[0] * 60 + pieces[1];
      }
    }
  }
  return undefined;
};

const formatDuration = (seconds?: number): string => {
  if (seconds === undefined || Number.isNaN(seconds) || seconds < 0) {
    return '--';
  }
  const totalSeconds = Math.floor(seconds);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const secs = totalSeconds % 60;
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}天`);
  if (hours > 0) parts.push(`${hours}小时`);
  if (minutes > 0) parts.push(`${minutes}分`);
  parts.push(`${secs}秒`);
  return parts.join(' ');
};

const formatDateTime = (date?: Date): string => {
  if (!date) {
    return '--';
  }
  return date.toLocaleString('zh-CN', {
    hour12: false,
  });
};

const formatCount = (value?: number): string => {
  return value !== undefined ? value.toLocaleString('zh-CN') : '--';
};

export function BackendStatus() {
  const { data, error } = useSWR<SystemStatus>('system-status', api.getStatus, {
    refreshInterval: STATUS_REFRESH_INTERVAL,
    revalidateOnFocus: false,
    dedupingInterval: 5000,
  });
  const isLoading = !data && !error;

  const isOnline = parseBooleanValue(pickFieldValue(data, STATUS_FIELD_KEYS.online));
  const runtimeSeconds = extractRuntimeSeconds(data);
  const lastUpdate = parseTimestampValue(pickFieldValue(data, STATUS_FIELD_KEYS.lastUpdate));
  const positionCount = parseCountValue(pickFieldValue(data, STATUS_FIELD_KEYS.positionCount));
  const tradeCount = parseCountValue(pickFieldValue(data, STATUS_FIELD_KEYS.tradeCount));
  const aiAnalysisCount = parseCountValue(pickFieldValue(data, STATUS_FIELD_KEYS.aiAnalysisCount));

  const statusLabel = isLoading
    ? '加载中...'
    : error
    ? '获取失败'
    : isOnline === undefined
    ? '未知'
    : isOnline
    ? '在线'
    : '离线';

  const statusTone = error
    ? 'error'
    : isOnline === undefined
    ? 'unknown'
    : isOnline
    ? 'online'
    : 'offline';

  const statusColorMap: Record<string, string> = {
    online: 'text-green-400',
    offline: 'text-red-400',
    unknown: 'text-yellow-400',
    error: 'text-red-400',
  };

  const statusDotMap: Record<string, string> = {
    online: 'bg-green-400',
    offline: 'bg-red-400',
    unknown: 'bg-yellow-400',
    error: 'bg-red-400',
  };

  const statusColor = statusColorMap[statusTone] ?? 'text-gray-400';
  const statusDotClass = statusDotMap[statusTone] ?? 'bg-gray-400';

  const metricItems = [
    {
      key: 'status',
      label: '运行状态',
      value: statusLabel,
      accent: statusColor,
    },
    {
      key: 'uptime',
      label: '运行时长',
      value: formatDuration(runtimeSeconds),
    },
    {
      key: 'updatedAt',
      label: '最后更新时间',
      value: formatDateTime(lastUpdate),
    },
    {
      key: 'positions',
      label: '持仓数',
      value: formatCount(positionCount),
    },
    {
      key: 'trades',
      label: '交易数',
      value: formatCount(tradeCount),
    },
    {
      key: 'ai',
      label: 'AI 分析数',
      value: formatCount(aiAnalysisCount),
    },
  ];

  return (
    <div className="binance-card border border-gray-800 px-6 py-5 rounded-xl shadow-lg">
      <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="text-sm binance-text-secondary">后端状态</p>
          <h2 className="text-lg font-semibold binance-text-primary">运行监控面板</h2>
          <p className="text-xs text-gray-500 mt-1">数据每10秒自动刷新</p>
        </div>
        <div className={`flex items-center gap-2 text-lg font-semibold ${statusColor}`}>
          <span className={`h-2.5 w-2.5 rounded-full ${statusDotClass}`} />
          {statusLabel}
        </div>
      </div>

      {error && (
        <p className="text-sm text-red-400 mt-3">
          获取后端状态失败，请检查 API 是否可用。
        </p>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-5">
        {metricItems.map((item) => (
          <div key={item.key}>
            <p className="text-sm binance-text-secondary">{item.label}</p>
            <p className={`text-xl font-semibold ${item.accent ?? 'binance-text-primary'}`}>
              {item.value}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
