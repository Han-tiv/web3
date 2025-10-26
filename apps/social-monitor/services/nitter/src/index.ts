import axios from 'axios';
import { createClient } from 'redis';
import winston from 'winston';
import cron from 'node-cron';
import { parseStringPromise } from 'xml2js';
import { TwitterFilters } from './filters';

interface Tweet {
  id: string;
  content: string;
  author: string;
  timestamp: Date;
  url: string;
  retweetsCount?: number;
  likesCount?: number;
}

interface CryptoOpportunity {
  id: string;
  type: 'red_packet' | 'airdrop' | 'giveaway' | 'alpha' | 'news';
  platform: 'twitter';
  source: string;
  title: string;
  description: string;
  requirements: string[];
  deadline?: Date;
  reward?: string;
  priority: number;
  url: string;
  timestamp: Date;
  metadata: {
    author: string;
    retweetsCount?: number;
    likesCount?: number;
    codes?: string[];
    mentions?: string[];
    hashtags?: string[];
  };
}

class NitterService {
  private nitterUrl: string;
  private redisClient: any;
  private logger!: winston.Logger;
  private monitoredAccounts: string[];
  private keywords: string[];
  private twitterFilters: TwitterFilters;
  private stats: {
    tweetsProcessed: number;
    opportunitiesFound: number;
    startTime: Date;
  };

  constructor() {
    this.nitterUrl = process.env.NITTER_URL || 'http://localhost:8080';
    this.twitterFilters = new TwitterFilters();
    this.stats = {
      tweetsProcessed: 0,
      opportunitiesFound: 0,
      startTime: new Date()
    };

    // Enhanced monitoring list with better categorization (Updated: removed suspended accounts)
    this.monitoredAccounts = [
      // Major Exchanges - Tier 1 (Active accounts only)
      'binance', 'coinbase', 'brian_armstrong',
      'okx', 'kucoincom', 'bybit_official', 'gate_io', 'bingxofficial',

      // DeFi Protocols - Tier 1
      'uniswap', 'aave', 'compound', 'makerdao', 'chainlink',

      // Major Projects - Tier 2
      'ethereum', 'bitcoin', 'opensea', 'metamask', 'trustwallet',

      // News/Analysis - Tier 2
      'coinmarketcap', 'coingecko', 'defipulse', 'theblock__', 'cointelegraph',

      // Influencers - Tier 3
      'crypto_gains', 'altcoindaily', 'coinbureau', 'investanswers',

      // Additional active accounts (替代被封账户)
      'pumpdotfun', 'solana', 'arbitrum', 'optimismfnd', 'base'
    ];

    // Focused keywords for better signal-to-noise ratio
    this.keywords = [
      // Red packets (highest priority)
      '红包', 'red packet', 'redpacket', 'bp', 'bpay',

      // Direct rewards
      'giveaway', 'airdrop', 'free crypto', 'claim', 'reward', 'bonus',

      // Chinese equivalents
      '空投', '福利', '抽奖', '币安红包',

      // Actionable opportunities
      'trading competition', 'learn and earn', 'quiz', 'contest'
    ];

    this.setupLogger();
    this.setupRedis();
  }

  private setupLogger(): void {
    this.logger = winston.createLogger({
      level: 'info',
      format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.errors({ stack: true }),
        winston.format.json()
      ),
      transports: [
        new winston.transports.File({
          filename: './logs/nitter-error.log',
          level: 'error'
        }),
        new winston.transports.File({
          filename: './logs/nitter.log'
        }),
        new winston.transports.Console({
          format: winston.format.combine(
            winston.format.colorize(),
            winston.format.simple()
          )
        })
      ]
    });
  }

  private async setupRedis(): Promise<void> {
    try {
      this.redisClient = createClient({
        url: process.env.REDIS_URL || 'redis://redis:6379'
      });

      this.redisClient.on('error', (err: Error) => {
        this.logger.error('Redis Client Error:', err);
      });

      await this.redisClient.connect();
      this.logger.info('Connected to Redis');
    } catch (error) {
      this.logger.error('Failed to connect to Redis:', error);
      process.exit(1);
    }
  }

  async start(): Promise<void> {
    this.logger.info('Starting Nitter monitoring service...');

    // Clean up any existing mock data on startup
    await this.cleanupMockData();

    // Monitor RSS feeds every 2 minutes
    cron.schedule('*/2 * * * *', async () => {
      await this.monitorAccounts();
    });

    // Monitor trending/search every 5 minutes
    cron.schedule('*/5 * * * *', async () => {
      await this.monitorKeywords();
    });

    // Health check every minute
    cron.schedule('* * * * *', async () => {
      await this.healthCheck();
    });

    this.logger.info('Nitter service started successfully');
  }

  private async cleanupMockData(): Promise<void> {
    try {
      const keys = await this.redisClient.keys('opportunity:*mock*');
      if (keys.length > 0) {
        await this.redisClient.del(keys);
        this.logger.info(`Cleaned up ${keys.length} mock data entries from Redis`);
      }
    } catch (error) {
      this.logger.error('Error cleaning up mock data:', error);
    }
  }

  private async monitorAccounts(): Promise<void> {
    for (const account of this.monitoredAccounts) {
      try {
        const tweets = await this.fetchAccountTweets(account);

        for (const tweet of tweets) {
          const opportunity = this.twitterFilters.analyzeOpportunity(tweet);
          if (opportunity) {
            await this.publishOpportunity(opportunity);
            this.stats.opportunitiesFound++;
          }
          this.stats.tweetsProcessed++;
        }

        // Rate limiting - wait 1 second between accounts
        await new Promise(resolve => setTimeout(resolve, 1000));
      } catch (error) {
        this.logger.error(`Error monitoring account ${account}:`, error);
      }
    }
  }

  private async fetchAccountTweets(account: string): Promise<Tweet[]> {
    try {
      const rssUrl = `${this.nitterUrl}/${account}/rss`;
      const response = await axios.get(rssUrl, { timeout: 10000 });

      const result = await parseStringPromise(response.data);
      const items = result.rss?.channel?.[0]?.item || [];

      return items.slice(0, 10).map((item: any) => ({
        id: this.extractTweetId(item.link?.[0]),
        content: this.cleanTweetContent(item.description?.[0] || ''),
        author: account,
        timestamp: new Date(item.pubDate?.[0]),
        url: item.link?.[0] || '',
      }));
    } catch (error) {
      this.logger.error(`Failed to fetch tweets for ${account}:`, error);
      return [];
    }
  }

  private extractTweetId(url: string): string {
    const match = url.match(/\/status\/(\d+)/);
    return match ? match[1] : '';
  }

  private cleanTweetContent(content: string): string {
    // Remove HTML tags and decode entities
    return content
      .replace(/<[^>]*>/g, '')
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .trim();
  }


  private async monitorKeywords(): Promise<void> {
    for (const keyword of this.keywords.slice(0, 5)) { // Monitor top 5 keywords to avoid rate limits
      try {
        const tweets = await this.searchTweets(keyword);

        for (const tweet of tweets) {
          const opportunity = this.twitterFilters.analyzeOpportunity(tweet);
          if (opportunity) {
            await this.publishOpportunity(opportunity);
            this.stats.opportunitiesFound++;
          }
          this.stats.tweetsProcessed++;
        }

        await new Promise(resolve => setTimeout(resolve, 2000));
      } catch (error) {
        this.logger.error(`Error searching keyword ${keyword}:`, error);
      }
    }
  }

  private async searchTweets(query: string): Promise<Tweet[]> {
    try {
      // Note: Nitter search might be limited, this is a basic implementation
      const searchUrl = `${this.nitterUrl}/search?q=${encodeURIComponent(query)}&f=tweets`;
      const response = await axios.get(searchUrl, { timeout: 10000 });

      // Parse HTML would be complex, for now return empty
      // In production, you'd want to parse the HTML response
      return [];
    } catch (error) {
      this.logger.error(`Error searching for ${query}:`, error);
      return [];
    }
  }

  private async publishOpportunity(opportunity: CryptoOpportunity): Promise<void> {
    try {
      // Check if already processed
      const exists = await this.redisClient.exists(`opportunity:${opportunity.id}`);
      if (exists) return;

      // Store in Redis with expiration
      await this.redisClient.setEx(`opportunity:${opportunity.id}`, 86400, JSON.stringify(opportunity));

      // Publish to aggregator
      await this.redisClient.publish('crypto_opportunities', JSON.stringify({
        ...opportunity,
        service: 'nitter'
      }));

      this.logger.info(`Published opportunity: ${opportunity.type} from ${opportunity.source} (Priority: ${opportunity.priority})`);
    } catch (error) {
      this.logger.error('Error publishing opportunity:', error);
    }
  }

  private async healthCheck(): Promise<void> {
    try {
      // Check Redis connection
      await this.redisClient.ping();

      // Log stats periodically
      const uptime = Date.now() - this.stats.startTime.getTime();
      this.logger.info('Health check passed', {
        uptime: `${Math.round(uptime / 1000 / 60)} minutes`,
        tweetsProcessed: this.stats.tweetsProcessed,
        opportunitiesFound: this.stats.opportunitiesFound,
        successRate: this.stats.tweetsProcessed > 0 ?
          Math.round((this.stats.opportunitiesFound / this.stats.tweetsProcessed) * 100) / 100 : 0
      });
    } catch (error) {
      this.logger.error('Health check failed:', error);
    }
  }

  // API endpoint for statistics
  getStats(): any {
    const uptime = Date.now() - this.stats.startTime.getTime();
    return {
      service: 'nitter',
      uptime: Math.round(uptime / 1000),
      stats: this.stats,
      monitoredAccounts: this.monitoredAccounts.length,
      keywords: this.keywords.length,
      filters: this.twitterFilters.getFilteringStats(),
      lastUpdate: new Date().toISOString()
    };
  }

  // Get recent opportunities for API
  async getRecentOpportunities(limit: number = 10): Promise<any[]> {
    try {
      const keys = await this.redisClient.keys('opportunity:*');
      const opportunities = [];

      for (const key of keys) {
        const data = await this.redisClient.get(key);
        if (data) {
          const opportunity = JSON.parse(data);
          // Filter out any mock data
          if (!opportunity.id?.includes('mock_') && !opportunity.title?.includes('BP-CODE123')) {
            opportunities.push(opportunity);
          }
        }
      }

      return opportunities
        .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
        .slice(0, limit);
    } catch (error) {
      this.logger.error('Error fetching recent opportunities:', error);
      return [];
    }
  }

  // Get monitored accounts for API
  getMonitoredAccounts(): string[] {
    return this.monitoredAccounts;
  }

  // Get keywords for API
  getKeywords(): string[] {
    return this.keywords;
  }
}

// Start the service
async function main() {
  const service = new NitterService();

  process.on('SIGINT', async () => {
    console.log('Shutting down Nitter service...');
    process.exit(0);
  });

  process.on('SIGTERM', async () => {
    console.log('Shutting down Nitter service...');
    process.exit(0);
  });

  await service.start();
}

if (require.main === module) {
  main().catch(console.error);
}

export { NitterService, CryptoOpportunity, Tweet };