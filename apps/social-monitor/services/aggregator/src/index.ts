// 社交媒体数据聚合服务

import express, { Application, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { createServer } from 'http';
import { WebSocketServer, WebSocket } from 'ws';
import Redis from 'redis';
import Winston from 'winston';
import cron from 'node-cron';
import dotenv from 'dotenv';

import { ValueScanClient } from './clients/valueScanClient';
import { TelegramNotifier } from './notifiers/telegramNotifier';
import { ValueScanWatcher } from './watchers/valueScanWatcher';

dotenv.config();

// 类型定义
interface CryptoOpportunity {
  id: string;
  type: 'airdrop' | 'giveaway' | 'redpacket' | 'learn_earn' | 'p2e';
  title: string;
  description: string;
  source: string;
  source_service: 'telegram' | 'discord' | 'nitter';
  url?: string;
  deadline?: Date;
  estimatedValue: number;
  priority: number;
  requirements?: string[];
  timestamp: Date;
  credibility_score?: number;
  spam_score?: number;
}

interface AggregatedData {
  opportunities: CryptoOpportunity[];
  stats: {
    total: number;
    by_type: Record<string, number>;
    by_source: Record<string, number>;
    high_priority: number;
    recent: number;
  };
  health: {
    services: Record<string, 'healthy' | 'warning' | 'error'>;
    last_update: Date;
  };
}

// 日志配置
const logger = Winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: Winston.format.combine(
    Winston.format.timestamp(),
    Winston.format.json()
  ),
  transports: [
    new Winston.transports.File({ filename: '/app/logs/aggregator-error.log', level: 'error' }),
    new Winston.transports.File({ filename: '/app/logs/aggregator.log' }),
    new Winston.transports.Console()
  ]
});

class SocialAggregator {
  private app: Application;
  private server: any;
  private wss: WebSocketServer;
  private redis: Redis.RedisClientType;
  private clients: Set<WebSocket> = new Set();
  private valueScanClient?: ValueScanClient;
  private telegramNotifier?: TelegramNotifier;
  private valueScanWatcher?: ValueScanWatcher;

  constructor() {
    this.app = express();
    this.server = createServer(this.app);
    this.wss = new WebSocketServer({ server: this.server });
    this.redis = Redis.createClient({ url: process.env.REDIS_URL });

    this.setupMiddleware();
    this.setupRoutes();
    this.setupWebSocket();
    this.setupRedisSubscriber();
  }

  async initialize(): Promise<void> {
    try {
      await this.redis.connect();

      this.setupValueScanWatcher();

      const port = process.env.PORT || 3002;
      this.server.listen(port, () => {
        logger.info(`社交媒体聚合服务已启动，端口: ${port}`);
      });

      this.startBackgroundTasks();
    } catch (error) {
      logger.error('初始化失败:', error);
      throw error;
    }
  }

  private setupMiddleware(): void {
    this.app.use(helmet());
    this.app.use(cors());
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use(express.urlencoded({ extended: true }));

    // 请求日志
    this.app.use((req, res, next) => {
      logger.info(`${req.method} ${req.path}`, {
        ip: req.ip,
        userAgent: req.get('User-Agent')
      });
      next();
    });
  }

  private setupValueScanWatcher(): void {
    const bearerToken = process.env.VALUESCAN_BEARER_TOKEN;
    const accessTicket = process.env.VALUESCAN_ACCESS_TICKET;

    if (!bearerToken || !accessTicket) {
      logger.warn('ValueScan 凭证缺失，跳过异动监控功能');
      return;
    }

    const botToken = process.env.TELEGRAM_BOT_TOKEN;
    const chatId = process.env.TELEGRAM_CHAT_ID;

    if (!botToken || !chatId) {
      logger.warn('Telegram 配置缺失，无法发送异动通知');
      return;
    }

    this.valueScanClient = new ValueScanClient({
      bearerToken,
      accessTicket,
      logger,
      pageSize: Number(process.env.VALUESCAN_PAGE_SIZE ?? 50)
    });

    this.telegramNotifier = new TelegramNotifier({
      botToken,
      chatId,
      logger,
      dryRun: process.env.TELEGRAM_DRY_RUN === 'true',
      disableNotification: process.env.TELEGRAM_SILENT === 'true'
    });

    this.valueScanWatcher = new ValueScanWatcher(
      this.redis,
      this.valueScanClient,
      this.telegramNotifier,
      logger,
      {
        minNumber24h: Number(process.env.VALUESCAN_MIN_TRIGGERS_24H ?? 0)
      }
    );

    logger.info('ValueScan 异动监控已启用');
  }

  private setupRoutes(): void {
    // 健康检查
    this.app.get('/health', (req: Request, res: Response) => {
      res.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        version: '1.0.0'
      });
    });

    // 获取所有机会
    this.app.get('/api/opportunities', async (req: Request, res: Response) => {
      try {
        const {
          type,
          source,
          minPriority = 0,
          limit = 50,
          offset = 0
        } = req.query;

        const opportunities = await this.getOpportunities({
          type: type as string,
          source: source as string,
          minPriority: parseInt(minPriority as string),
          limit: parseInt(limit as string),
          offset: parseInt(offset as string)
        });

        res.json({
          success: true,
          data: opportunities,
          total: opportunities.length,
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('获取机会失败:', error);
        res.status(500).json({
          success: false,
          error: 'Internal server error'
        });
      }
    });

    // 获取聚合统计数据
    this.app.get('/api/dashboard', async (req: Request, res: Response) => {
      try {
        const data = await this.getAggregatedData();
        res.json({
          success: true,
          data,
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('获取聚合数据失败:', error);
        res.status(500).json({
          success: false,
          error: 'Internal server error'
        });
      }
    });

    // 获取高优先级机会
    this.app.get('/api/opportunities/priority', async (req: Request, res: Response) => {
      try {
        const highPriorityIds = await this.redis.zRevRange('opportunities:priority', 0, 19);
        const opportunities: CryptoOpportunity[] = [];

        for (const id of highPriorityIds) {
          const data = await this.redis.get(`opportunity:${id}`);
          if (data) {
            opportunities.push(JSON.parse(data));
          }
        }

        res.json({
          success: true,
          data: opportunities.filter(opp => opp.priority >= 7),
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('获取高优先级机会失败:', error);
        res.status(500).json({
          success: false,
          error: 'Internal server error'
        });
      }
    });

    // 获取实时统计
    this.app.get('/api/stats/realtime', async (req: Request, res: Response) => {
      try {
        const stats = await this.getRealTimeStats();
        res.json({
          success: true,
          data: stats,
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('获取实时统计失败:', error);
        res.status(500).json({
          success: false,
          error: 'Internal server error'
        });
      }
    });

    // 手动触发数据同步
    this.app.post('/api/sync', async (req: Request, res: Response) => {
      try {
        await this.syncAllData();
        res.json({
          success: true,
          message: '数据同步已触发',
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('手动同步失败:', error);
        res.status(500).json({
          success: false,
          error: 'Sync failed'
        });
      }
    });

    // 手动触发 ValueScan 异动检测
    this.app.post('/api/valuescan/scan', async (_req: Request, res: Response) => {
      if (!this.valueScanWatcher) {
        return res.status(503).json({
          success: false,
          error: 'ValueScan watcher not configured'
        });
      }

      try {
        await this.valueScanWatcher.run();
        res.json({
          success: true,
          message: 'ValueScan watcher executed',
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('手动触发 ValueScan 失败:', error);
        res.status(500).json({
          success: false,
          error: 'ValueScan scan failed'
        });
      }
    });
  }

  private setupWebSocket(): void {
    this.wss.on('connection', (ws: WebSocket, req) => {
      logger.info('新的WebSocket连接', { ip: req.socket.remoteAddress });
      this.clients.add(ws);

      // 发送欢迎消息
      ws.send(JSON.stringify({
        type: 'connected',
        message: '已连接到社交媒体聚合服务',
        timestamp: new Date().toISOString()
      }));

      ws.on('close', () => {
        this.clients.delete(ws);
        logger.info('WebSocket连接关闭');
      });

      ws.on('error', (error) => {
        logger.error('WebSocket错误:', error);
        this.clients.delete(ws);
      });
    });
  }

  private setupRedisSubscriber(): void {
    // 创建订阅客户端
    const subscriber = this.redis.duplicate();
    subscriber.connect();

    // 订阅新机会事件
    subscriber.subscribe('new_opportunity', (message) => {
      try {
        const opportunity: CryptoOpportunity = JSON.parse(message);
        logger.info('收到新机会:', {
          id: opportunity.id,
          priority: opportunity.priority,
          source: opportunity.source_service
        });

        // 广播到所有WebSocket客户端
        this.broadcast({
          type: 'new_opportunity',
          data: opportunity,
          timestamp: new Date().toISOString()
        });

        // 如果是高优先级机会，发送特殊通知
        if (opportunity.priority >= 8) {
          this.broadcast({
            type: 'high_priority_opportunity',
            data: opportunity,
            timestamp: new Date().toISOString()
          });
        }
      } catch (error) {
        logger.error('处理新机会事件失败:', error);
      }
    });

    // 订阅系统状态更新
    subscriber.subscribe('service_status', (message) => {
      try {
        const status = JSON.parse(message);
        this.broadcast({
          type: 'service_status',
          data: status,
          timestamp: new Date().toISOString()
        });
      } catch (error) {
        logger.error('处理服务状态更新失败:', error);
      }
    });
  }

  private broadcast(data: any): void {
    const message = JSON.stringify(data);
    this.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(message);
      }
    });
  }

  private async getOpportunities(filters: {
    type?: string;
    source?: string;
    minPriority: number;
    limit: number;
    offset: number;
  }): Promise<CryptoOpportunity[]> {
    const { type, source, minPriority, limit, offset } = filters;
    const opportunities: CryptoOpportunity[] = [];

    // 从Redis获取所有机会ID
    const allIds = await this.redis.zRevRange('opportunities:priority', offset, offset + limit - 1);

    for (const id of allIds) {
      const data = await this.redis.get(`opportunity:${id}`);
      if (data) {
        const opportunity: CryptoOpportunity = JSON.parse(data);

        // 应用过滤器
        if (type && opportunity.type !== type) continue;
        if (source && opportunity.source_service !== source) continue;
        if (opportunity.priority < minPriority) continue;

        opportunities.push(opportunity);
      }
    }

    return opportunities;
  }

  private async getAggregatedData(): Promise<AggregatedData> {
    const opportunities = await this.getOpportunities({
      minPriority: 0,
      limit: 1000,
      offset: 0
    });

    // 计算统计数据
    const stats = {
      total: opportunities.length,
      by_type: {} as Record<string, number>,
      by_source: {} as Record<string, number>,
      high_priority: opportunities.filter(o => o.priority >= 8).length,
      recent: opportunities.filter(o =>
        new Date(o.timestamp).getTime() > Date.now() - 24 * 60 * 60 * 1000
      ).length
    };

    opportunities.forEach(opp => {
      stats.by_type[opp.type] = (stats.by_type[opp.type] || 0) + 1;
      stats.by_source[opp.source_service] = (stats.by_source[opp.source_service] || 0) + 1;
    });

    // 检查服务健康状态
    const health = {
      services: {
        telegram: await this.checkServiceHealth('telegram'),
        discord: await this.checkServiceHealth('discord'),
        nitter: await this.checkServiceHealth('nitter')
      },
      last_update: new Date()
    };

    return {
      opportunities,
      stats,
      health
    };
  }

  private async checkServiceHealth(service: string): Promise<'healthy' | 'warning' | 'error'> {
    try {
      const lastActivity = await this.redis.get(`service:${service}:last_activity`);
      if (!lastActivity) return 'error';

      const lastTime = new Date(lastActivity).getTime();
      const now = Date.now();
      const diff = now - lastTime;

      if (diff < 5 * 60 * 1000) return 'healthy';  // 5分钟内
      if (diff < 15 * 60 * 1000) return 'warning'; // 15分钟内
      return 'error';
    } catch (error) {
      return 'error';
    }
  }

  private async getRealTimeStats(): Promise<any> {
    const now = new Date();
    const hourAgo = new Date(now.getTime() - 60 * 60 * 1000);

    const recentOpportunities = await this.getOpportunities({
      minPriority: 0,
      limit: 1000,
      offset: 0
    });

    const recentCount = recentOpportunities.filter(o =>
      new Date(o.timestamp) >= hourAgo
    ).length;

    return {
      active_connections: this.clients.size,
      opportunities_last_hour: recentCount,
      redis_connections: await this.redis.clientInfo(),
      memory_usage: process.memoryUsage(),
      uptime: process.uptime()
    };
  }

  private async syncAllData(): Promise<void> {
    logger.info('开始数据同步...');

    // 清理过期数据
    const expiredIds = await this.redis.keys('opportunity:*');
    for (const key of expiredIds) {
      const data = await this.redis.get(key);
      if (data) {
        const opportunity: CryptoOpportunity = JSON.parse(data);
        if (opportunity.deadline && new Date(opportunity.deadline) < new Date()) {
          await this.redis.del(key);
          await this.redis.zRem('opportunities:priority', opportunity.id);
        }
      }
    }

    logger.info('数据同步完成');
  }

  private startBackgroundTasks(): void {
    // 每5分钟同步一次数据
    cron.schedule('*/5 * * * *', async () => {
      await this.syncAllData();
    });

    // 每30分钟发送统计数据
    cron.schedule('*/30 * * * *', async () => {
      const stats = await this.getRealTimeStats();
      this.broadcast({
        type: 'stats_update',
        data: stats,
        timestamp: new Date().toISOString()
      });
    });

    // 每小时清理无效连接
    cron.schedule('0 * * * *', () => {
      this.clients.forEach(client => {
        if (client.readyState !== WebSocket.OPEN) {
          this.clients.delete(client);
        }
      });
      logger.info(`清理后活跃连接数: ${this.clients.size}`);
    });

    if (this.valueScanWatcher) {
      const cronExpression = process.env.VALUESCAN_CRON || '*/2 * * * *';
      cron.schedule(cronExpression, async () => {
        try {
          await this.valueScanWatcher?.run();
        } catch (error) {
          logger.error('执行 ValueScan 异动轮询失败', { error });
        }
      });
      logger.info(`ValueScan 轮询已启用，Cron: ${cronExpression}`);
    }

    logger.info('后台任务已启动');
  }

  async shutdown(): Promise<void> {
    logger.info('正在关闭聚合服务...');

    this.clients.forEach(client => {
      client.close();
    });

    this.server.close();
    await this.redis.quit();
  }
}

// 主程序入口
const aggregator = new SocialAggregator();

process.on('SIGINT', async () => {
  await aggregator.shutdown();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await aggregator.shutdown();
  process.exit(0);
});

// 启动服务
aggregator.initialize().catch((error) => {
  logger.error('服务启动失败:', error);
  process.exit(1);
});
