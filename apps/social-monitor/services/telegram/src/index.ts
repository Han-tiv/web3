// Telegram监控服务主入口

import { Telegraf, Context } from 'telegraf';
import { TelegramApi } from 'telegram';
import { StringSession } from 'telegram/sessions';
import Redis from 'redis';
import Winston from 'winston';
import cron from 'node-cron';
import dotenv from 'dotenv';

dotenv.config();

// 日志配置
const logger = Winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: Winston.format.combine(
    Winston.format.timestamp(),
    Winston.format.json()
  ),
  transports: [
    new Winston.transports.File({ filename: '/app/logs/telegram-error.log', level: 'error' }),
    new Winston.transports.File({ filename: '/app/logs/telegram.log' }),
    new Winston.transports.Console()
  ]
});

interface CryptoOpportunity {
  id: string;
  type: 'airdrop' | 'giveaway' | 'redpacket' | 'learn_earn' | 'p2e';
  title: string;
  description: string;
  source: string;
  url?: string;
  deadline?: Date;
  estimatedValue: number;
  priority: number;
  requirements?: string[];
  timestamp: Date;
}

class TelegramMonitor {
  private bot: Telegraf;
  private client: TelegramApi;
  private redis: Redis.RedisClientType;
  private monitoredGroups: string[] = [];
  private keywords: string[] = [];

  constructor() {
    // 初始化Bot
    this.bot = new Telegraf(process.env.TELEGRAM_BOT_TOKEN!);

    // 初始化Client (用于监控群组)
    const session = new StringSession(process.env.TELEGRAM_SESSION_STRING || '');
    this.client = new TelegramApi(session, parseInt(process.env.TELEGRAM_API_ID!), process.env.TELEGRAM_API_HASH!);

    // 初始化Redis
    this.redis = Redis.createClient({ url: process.env.REDIS_URL });

    // 配置监控目标
    this.monitoredGroups = (process.env.TELEGRAM_GROUPS || '').split(',').filter(g => g.trim());
    this.keywords = (process.env.TELEGRAM_KEYWORDS || 'airdrop,giveaway,红包,空投').split(',').map(k => k.trim());

    this.setupEventHandlers();
  }

  async initialize(): Promise<void> {
    try {
      await this.redis.connect();
      await this.client.start({ botAuthToken: process.env.TELEGRAM_BOT_TOKEN! });
      logger.info('Telegram监控服务已启动');

      // 启动监控任务
      this.startMonitoring();
    } catch (error) {
      logger.error('初始化失败:', error);
      throw error;
    }
  }

  private setupEventHandlers(): void {
    // Bot消息处理
    this.bot.on('message', async (ctx) => {
      await this.processMessage(ctx);
    });

    // Client消息监听
    this.client.addEventHandler(async (update) => {
      if (update.className === 'UpdateNewMessage') {
        await this.processChannelMessage(update);
      }
    });
  }

  private async processMessage(ctx: Context): Promise<void> {
    try {
      const message = ctx.message;
      if (!message || !('text' in message)) return;

      const opportunity = await this.analyzeMessage({
        text: message.text,
        chatId: message.chat.id,
        messageId: message.message_id,
        date: new Date(message.date * 1000),
        from: message.from?.username || 'unknown'
      });

      if (opportunity) {
        await this.saveOpportunity(opportunity);
        logger.info('发现新机会:', { title: opportunity.title, priority: opportunity.priority });
      }
    } catch (error) {
      logger.error('处理消息失败:', error);
    }
  }

  private async processChannelMessage(update: any): Promise<void> {
    try {
      const message = update.message;
      if (!message || !message.message) return;

      const opportunity = await this.analyzeMessage({
        text: message.message,
        chatId: message.peerId.channelId,
        messageId: message.id,
        date: new Date(message.date * 1000),
        from: 'channel'
      });

      if (opportunity) {
        await this.saveOpportunity(opportunity);
        logger.info('频道发现新机会:', { title: opportunity.title, priority: opportunity.priority });
      }
    } catch (error) {
      logger.error('处理频道消息失败:', error);
    }
  }

  private async analyzeMessage(messageData: {
    text: string;
    chatId: number;
    messageId: number;
    date: Date;
    from: string;
  }): Promise<CryptoOpportunity | null> {
    const { text, chatId, messageId, date, from } = messageData;
    const lowerText = text.toLowerCase();

    // 关键词匹配
    const matchedKeywords = this.keywords.filter(keyword =>
      lowerText.includes(keyword.toLowerCase())
    );

    if (matchedKeywords.length === 0) return null;

    // 识别机会类型
    let type: CryptoOpportunity['type'] = 'airdrop';
    let priority = 5;

    if (lowerText.includes('红包') || lowerText.includes('red packet')) {
      type = 'redpacket';
      priority = 8;
    } else if (lowerText.includes('giveaway') || lowerText.includes('抽奖')) {
      type = 'giveaway';
      priority = 6;
    } else if (lowerText.includes('learn') || lowerText.includes('学习')) {
      type = 'learn_earn';
      priority = 4;
    } else if (lowerText.includes('game') || lowerText.includes('play') || lowerText.includes('游戏')) {
      type = 'p2e';
      priority = 5;
    }

    // 提取价值信息
    const valueRegex = /\$(\d+(?:\.\d+)?)|(\d+(?:\.\d+)?)\s*(usdt|busd|eth|btc)/gi;
    const valueMatches = text.match(valueRegex);
    const estimatedValue = this.calculateEstimatedValue(valueMatches);

    // 检测时间限制
    const deadline = this.extractDeadline(text);
    if (deadline && deadline < new Date()) {
      return null; // 过期机会
    }

    // 检测要求
    const requirements = this.extractRequirements(text);

    const opportunity: CryptoOpportunity = {
      id: `tg_${chatId}_${messageId}_${Date.now()}`,
      type,
      title: this.generateTitle(text, type),
      description: text.substring(0, 300) + (text.length > 300 ? '...' : ''),
      source: `telegram_${from}`,
      estimatedValue,
      priority: Math.min(10, priority + (estimatedValue > 100 ? 2 : 0)),
      requirements,
      timestamp: date,
      deadline
    };

    return opportunity;
  }

  private calculateEstimatedValue(matches: string[] | null): number {
    if (!matches) return 10;

    let totalValue = 0;
    matches.forEach(match => {
      const value = parseFloat(match.replace(/[^0-9.]/g, ''));
      if (match.toLowerCase().includes('btc')) {
        totalValue += value * 35000; // 假设BTC价格
      } else if (match.toLowerCase().includes('eth')) {
        totalValue += value * 1800; // 假设ETH价格
      } else {
        totalValue += value; // USDT等稳定币
      }
    });

    return Math.max(10, totalValue);
  }

  private extractDeadline(text: string): Date | undefined {
    // 简单的时间提取逻辑
    const timeRegex = /(\d{1,2}):(\d{2})|(\d{1,2})月(\d{1,2})日|(\d{4}-\d{2}-\d{2})/g;
    const matches = text.match(timeRegex);

    if (matches && matches.length > 0) {
      // 这里需要更复杂的时间解析逻辑
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      return tomorrow;
    }

    return undefined;
  }

  private extractRequirements(text: string): string[] {
    const requirements: string[] = [];

    if (text.includes('关注') || text.includes('follow')) {
      requirements.push('follow');
    }
    if (text.includes('转发') || text.includes('retweet') || text.includes('rt')) {
      requirements.push('retweet');
    }
    if (text.includes('点赞') || text.includes('like')) {
      requirements.push('like');
    }
    if (text.includes('加群') || text.includes('join')) {
      requirements.push('join_group');
    }
    if (text.includes('kyc')) {
      requirements.push('kyc');
    }

    return requirements;
  }

  private generateTitle(text: string, type: CryptoOpportunity['type']): string {
    const firstLine = text.split('\n')[0].substring(0, 50);
    const typeMap = {
      'airdrop': '空投',
      'giveaway': '抽奖',
      'redpacket': '红包',
      'learn_earn': '学习赚钱',
      'p2e': 'P2E游戏'
    };

    return `${typeMap[type]} - ${firstLine}`;
  }

  private async saveOpportunity(opportunity: CryptoOpportunity): Promise<void> {
    try {
      // 保存到Redis
      const key = `opportunity:${opportunity.id}`;
      await this.redis.setEx(key, 86400, JSON.stringify(opportunity)); // 24小时过期

      // 添加到优先级队列
      await this.redis.zAdd('opportunities:priority', {
        score: opportunity.priority,
        value: opportunity.id
      });

      // 发布事件通知
      await this.redis.publish('new_opportunity', JSON.stringify({
        ...opportunity,
        source_service: 'telegram'
      }));

    } catch (error) {
      logger.error('保存机会失败:', error);
    }
  }

  private startMonitoring(): void {
    // 启动定时任务
    cron.schedule('*/30 * * * * *', async () => {
      await this.healthCheck();
    });

    cron.schedule('0 * * * *', async () => {
      await this.cleanupExpiredOpportunities();
    });

    logger.info('定时监控任务已启动');
  }

  private async healthCheck(): Promise<void> {
    try {
      await this.redis.ping();
      // 可以添加更多健康检查逻辑
    } catch (error) {
      logger.error('健康检查失败:', error);
    }
  }

  private async cleanupExpiredOpportunities(): Promise<void> {
    try {
      const expiredKeys = await this.redis.keys('opportunity:*');
      // 清理过期的机会
      logger.info(`清理了 ${expiredKeys.length} 个过期机会`);
    } catch (error) {
      logger.error('清理过期机会失败:', error);
    }
  }

  async shutdown(): Promise<void> {
    logger.info('正在关闭Telegram监控服务...');
    await this.redis.quit();
    await this.client.disconnect();
    this.bot.stop();
  }
}

// 主程序入口
const telegramMonitor = new TelegramMonitor();

process.on('SIGINT', async () => {
  await telegramMonitor.shutdown();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await telegramMonitor.shutdown();
  process.exit(0);
});

// 启动服务
telegramMonitor.initialize().catch((error) => {
  logger.error('服务启动失败:', error);
  process.exit(1);
});