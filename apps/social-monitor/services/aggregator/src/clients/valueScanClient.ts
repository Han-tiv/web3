import axios, { AxiosInstance } from 'axios';
import { Logger } from 'winston';

export interface ValueScanFundsMovement {
  id: string;
  updateTime: number;
  tradeType: number;
  symbol: string;
  beginTime: number;
  endTime: number;
  number24h: number;
  numberNot24h: number;
  price: number;
  beginPrice: number;
  gains: number;
  decline: number;
  percentChange24h: number;
  marketCap: number;
  alpha: boolean;
  fomo: boolean;
  icon?: string;
}

interface RawFundsMovementItem {
  id: string;
  updateTime: number;
  tradeType: number;
  keyword: number | string;
  symbol: string;
  beginTime: number;
  endTime: number;
  number24h: number;
  numberNot24h: number;
  price: string;
  beginPrice: string;
  gains: number;
  decline: number;
  favor: boolean;
  percentChange24h: string;
  marketCap: string;
  observe: boolean;
  alpha: boolean | string;
  fomo: boolean | string;
  icon?: string;
}

interface ValueScanClientOptions {
  bearerToken: string;
  accessTicket: string;
  logger: Logger;
  pageSize?: number;
}

export class ValueScanClient {
  private readonly axios: AxiosInstance;
  private readonly logger: Logger;
  private readonly pageSize: number;

  constructor(options: ValueScanClientOptions) {
    const { bearerToken, accessTicket, logger, pageSize = 50 } = options;

    this.axios = axios.create({
      baseURL: 'https://api.valuescan.io',
      timeout: 10_000,
      headers: {
        Authorization: `Bearer ${bearerToken}`,
        'Access-Ticket': accessTicket,
        'Content-Type': 'application/json'
      }
    });

    this.logger = logger;
    this.pageSize = pageSize;
  }

  /**
   * 从 ValueScan 获取资金异动列表并进行字段标准化
   */
  async fetchFundsMovement(pageNum = 1): Promise<ValueScanFundsMovement[]> {
    try {
      const response = await this.axios.post('/api/chance/getFundsMovementPage', {
        pageNum,
        pageSize: this.pageSize
      });

      const { code, msg, data } = response.data ?? {};

      if (code !== 200) {
        this.logger.error('ValueScan 返回错误', { code, msg });
        throw new Error(`ValueScan error code ${code}: ${msg}`);
      }

      const list: RawFundsMovementItem[] = data?.list ?? [];
      return list.map((item) => this.normalizeItem(item));
    } catch (error) {
      this.logger.error('获取 ValueScan 异动数据失败', { error });
      throw error;
    }
  }

  private normalizeItem(item: RawFundsMovementItem): ValueScanFundsMovement {
    return {
      id: item.id,
      updateTime: item.updateTime,
      tradeType: item.tradeType,
      symbol: item.symbol,
      beginTime: item.beginTime,
      endTime: item.endTime,
      number24h: item.number24h,
      numberNot24h: item.numberNot24h,
      price: Number(item.price),
      beginPrice: Number(item.beginPrice),
      gains: Number(item.gains ?? 0),
      decline: Number(item.decline ?? 0),
      percentChange24h: Number(item.percentChange24h ?? 0),
      marketCap: Number(item.marketCap ?? 0),
      alpha: toBoolean(item.alpha),
      fomo: toBoolean(item.fomo),
      icon: item.icon
    };
  }
}

export function toBoolean(value: unknown): boolean {
  if (typeof value === 'boolean') {
    return value;
  }
  if (typeof value === 'number') {
    return value !== 0;
  }
  if (typeof value === 'string') {
    const trimmed = value.trim();
    if (!trimmed) {
      return false;
    }
    const normalized = trimmed.toLowerCase();
    return ['true', '1', 'yes', 'y', 'on', 'alpha', 'fomo'].includes(normalized);
  }
  return false;
}
