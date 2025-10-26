import { RedisClientType } from 'redis';
import { Logger } from 'winston';

import type { ValueScanFundsMovement } from '../clients/valueScanClient';

interface FundsMovementProvider {
  fetchFundsMovement(pageNum?: number): Promise<ValueScanFundsMovement[]>;
}

interface MessageSender {
  sendMessage(text: string): Promise<void>;
}

interface ValueScanWatcherOptions {
  redisKey?: string;
  expireSeconds?: number;
  minNumber24h?: number;
}

/**
 * 负责轮询 ValueScan 异动数据并在命中 alpha/fomo 标签时推送到 Telegram。
 */
export class ValueScanWatcher {
  private readonly redis: RedisClientType;
  private readonly client: FundsMovementProvider;
  private readonly notifier: MessageSender;
  private readonly logger: Logger;
  private readonly redisKey: string;
  private readonly expireSeconds: number;
  private readonly minNumber24h: number;

  constructor(
    redis: RedisClientType,
    client: FundsMovementProvider,
    notifier: MessageSender,
    logger: Logger,
    options: ValueScanWatcherOptions = {}
  ) {
    this.redis = redis;
    this.client = client;
    this.notifier = notifier;
    this.logger = logger;
    this.redisKey = options.redisKey ?? 'valuescan:funds:alerted';
    this.expireSeconds = options.expireSeconds ?? 24 * 60 * 60; // 默认保留24小时
    this.minNumber24h = options.minNumber24h ?? 0;
  }

  async run(): Promise<void> {
    const movements = await this.client.fetchFundsMovement();

    for (const movement of movements) {
      if (!this.shouldAlert(movement)) {
        continue;
      }

      if (movement.number24h < this.minNumber24h) {
        continue;
      }

      const dedupKey = `${movement.id}:${movement.updateTime}`;
      const alreadySent = await this.redis.sIsMember(this.redisKey, dedupKey);
      if (alreadySent) {
        continue;
      }

      const message = buildTelegramMessage(movement);
      try {
        await this.notifier.sendMessage(message);
        await this.redis.sAdd(this.redisKey, dedupKey);
        await this.redis.expire(this.redisKey, this.expireSeconds);
        this.logger.info('已推送 ValueScan 异动', {
          id: movement.id,
          symbol: movement.symbol,
          alpha: movement.alpha,
          fomo: movement.fomo
        });
      } catch (error) {
        this.logger.error('推送 ValueScan 异动失败', { error });
      }
    }
  }

  private shouldAlert(item: ValueScanFundsMovement): boolean {
    return Boolean(item.alpha || item.fomo);
  }
}

export function buildTelegramMessage(item: ValueScanFundsMovement): string {
  const tags: string[] = [];
  if (item.alpha) tags.push('alpha');
  if (item.fomo) tags.push('fomo');

  const percentFormatter = new Intl.NumberFormat('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  });

  const numberFormatter = new Intl.NumberFormat('zh-CN', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 0
  });

  const priceFormatter = new Intl.NumberFormat('zh-CN', {
    minimumFractionDigits: 4,
    maximumFractionDigits: 8
  });

  const lastUpdate = formatTimestamp(item.updateTime);
  const beginTime = formatTimestamp(item.beginTime);

  const percentChange = percentFormatter.format(item.percentChange24h);

  const number24h = numberFormatter.format(item.number24h);
  const numberNot24h = numberFormatter.format(item.numberNot24h);

  const price = priceFormatter.format(item.price);
  const beginPrice = priceFormatter.format(item.beginPrice);

  const marketCapFormatter = new Intl.NumberFormat('zh-CN', {
    notation: 'compact',
    maximumFractionDigits: 2
  });

  const marketCap = marketCapFormatter.format(item.marketCap);

  const direction = item.percentChange24h >= 0 ? '上涨' : '下跌';

  return [
    '🚨 <b>ValueScan 资金异动</b>',
    `📌 代币：<code>${item.symbol}</code>`,
    `🏷️ 标签：${tags.join(' / ') || '无'}`,
    `📊 24小时${direction}：<b>${percentChange}%</b>`,
    `🔁 异动统计：小周期 ${number24h} 次 / 大周期 ${numberNot24h} 次`,
    `💰 价格：${price}（起始 ${beginPrice}）`,
    `💹 市值：${marketCap}`,
    `🕒 周期：${beginTime} → ${lastUpdate}`
  ].join('\n');
}

function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
    timeZone: 'Asia/Shanghai'
  }).format(date);
}
