import { Tweet, CryptoOpportunity } from './index';

interface FilterRule {
  name: string;
  type: CryptoOpportunity['type'];
  keywords: string[];
  requiredKeywords?: string[];
  excludeKeywords?: string[];
  priorityBoost: number;
  accountBoost?: { [account: string]: number };
  timeConstraints?: {
    maxAgeHours: number;
    urgencyKeywords: string[];
  };
}

interface AccountTier {
  tier: 'premium' | 'verified' | 'standard';
  accounts: string[];
  priorityMultiplier: number;
  trustScore: number;
}

export class TwitterFilters {
  private filterRules!: FilterRule[];
  private accountTiers!: AccountTier[];
  private suspiciousPatterns!: RegExp[];
  private codePatterns!: RegExp[];

  constructor() {
    this.initializeFilterRules();
    this.initializeAccountTiers();
    this.initializeSuspiciousPatterns();
    this.initializeCodePatterns();
  }

  private initializeFilterRules(): void {
    this.filterRules = [
      // Red Packet Rules - Highest Priority
      {
        name: 'binance_red_packet',
        type: 'red_packet',
        keywords: ['红包', 'red packet', 'redpacket', 'bp', 'bpay'],
        requiredKeywords: ['binance', '币安'],
        excludeKeywords: ['scam', '骗局', 'fake', 'expired'],
        priorityBoost: 10,
        accountBoost: {
          'binance': 3,
          'cz_binance': 3,
          'binancechain': 2
        },
        timeConstraints: {
          maxAgeHours: 2, // Red packets expire quickly
          urgencyKeywords: ['first', '先到先得', 'limited', 'quick']
        }
      },
      {
        name: 'exchange_red_packet',
        type: 'red_packet',
        keywords: ['红包', 'red packet', 'redpacket', 'gift'],
        requiredKeywords: ['okx', 'bybit', 'kucoin', 'gate', 'mexc', 'bingx'],
        excludeKeywords: ['scam', '骗局', 'fake'],
        priorityBoost: 8,
        timeConstraints: {
          maxAgeHours: 4,
          urgencyKeywords: ['expires', 'limited time', '限时']
        }
      },

      // Airdrop Rules - High Priority
      {
        name: 'verified_airdrop',
        type: 'airdrop',
        keywords: ['airdrop', '空投', 'free tokens', 'claim'],
        requiredKeywords: [],
        excludeKeywords: ['scam', 'fake', '骗局', 'phishing'],
        priorityBoost: 7,
        accountBoost: {
          'coinbase': 2,
          'uniswap': 2,
          'opensea': 2,
          'metamask': 2
        },
        timeConstraints: {
          maxAgeHours: 48,
          urgencyKeywords: ['snapshot', 'deadline', '截止']
        }
      },

      // Giveaway Rules - Medium Priority
      {
        name: 'exchange_giveaway',
        type: 'giveaway',
        keywords: ['giveaway', '抽奖', 'contest', 'competition', 'lottery'],
        requiredKeywords: [],
        excludeKeywords: ['scam', 'fake', '骗局'],
        priorityBoost: 6,
        timeConstraints: {
          maxAgeHours: 72,
          urgencyKeywords: ['ends soon', '即将结束', 'final hours']
        }
      },

      // Alpha/Trading Signals - Lower Priority
      {
        name: 'trading_alpha',
        type: 'alpha',
        keywords: ['alpha', 'gem', 'moonshot', '100x', 'early', 'presale'],
        excludeKeywords: ['scam', 'rug', 'fake', '骗局'],
        priorityBoost: 4,
        timeConstraints: {
          maxAgeHours: 24,
          urgencyKeywords: ['breaking', '突发', 'just launched']
        }
      },

      // Learn and Earn - Consistent Income
      {
        name: 'learn_earn',
        type: 'alpha',
        keywords: ['learn and earn', 'quiz', 'education', 'course'],
        requiredKeywords: ['binance', 'coinbase', 'okx', 'bybit'],
        priorityBoost: 5,
        timeConstraints: {
          maxAgeHours: 168, // 1 week
          urgencyKeywords: []
        }
      }
    ];
  }

  private initializeAccountTiers(): void {
    this.accountTiers = [
      {
        tier: 'premium',
        accounts: [
          'binance', 'cz_binance', 'binancechain', 'coinbase', 'brian_armstrong',
          'okx', 'justinsuntron', 'ethereum', 'bitcoin', 'uniswap',
          'opensea', 'metamask', 'trustwallet', 'ledger'
        ],
        priorityMultiplier: 2.0,
        trustScore: 10
      },
      {
        tier: 'verified',
        accounts: [
          'kucoincom', 'bybit_official', 'gate_io', 'mexc_global', 'bingxofficial',
          'coinmarketcap', 'coingecko', 'defipulse', 'theblock__', 'cointelegraph'
        ],
        priorityMultiplier: 1.5,
        trustScore: 8
      },
      {
        tier: 'standard',
        accounts: [
          'crypto_gains', 'altcoindaily', 'coinbureau', 'investanswers',
          'pentosh1', 'rektcapital', 'crypto_bitlord'
        ],
        priorityMultiplier: 1.2,
        trustScore: 6
      }
    ];
  }

  private initializeSuspiciousPatterns(): void {
    this.suspiciousPatterns = [
      // Common scam patterns
      /send.*\d+.*get.*\d+.*back/i,
      /double.*your.*crypto/i,
      /guaranteed.*profit/i,
      /risk.*free/i,
      /click.*here.*free/i,
      /dm.*for.*details/i,
      /投资.*保证.*收益/i,
      /免费.*送.*币/i,
      /点击.*立即.*获得/i
    ];
  }

  private initializeCodePatterns(): void {
    this.codePatterns = [
      /\b[A-Z0-9]{8,12}\b/g, // General codes
      /BP[A-Z0-9]{6,10}/g, // Binance Pay codes
      /\b[A-Z]{4}[0-9]{4}[A-Z0-9]{4}\b/g, // Structured codes
      /红包码[：:]\s*([A-Z0-9]+)/g, // Chinese red packet codes
      /code[：:]\s*([A-Z0-9]+)/gi, // Code patterns
    ];
  }

  analyzeOpportunity(tweet: Tweet): CryptoOpportunity | null {
    const content = tweet.content.toLowerCase();

    // Skip if too old
    const maxAge = this.getMaxAgeForContent(content);
    if (Date.now() - tweet.timestamp.getTime() > maxAge * 60 * 60 * 1000) {
      return null;
    }

    // Check for suspicious patterns
    if (this.isSuspicious(tweet.content)) {
      return null;
    }

    // Find matching rule
    const rule = this.findMatchingRule(content, tweet.author);
    if (!rule) return null;

    // Calculate priority
    const priority = this.calculatePriority(tweet, rule);

    // Extract metadata
    const metadata = this.extractMetadata(tweet);

    // Extract requirements
    const requirements = this.extractRequirements(tweet.content);

    // Extract deadline and reward
    const deadline = this.extractDeadline(tweet.content);
    const reward = this.extractReward(tweet.content);

    return {
      id: `twitter_${tweet.id}`,
      type: rule.type,
      platform: 'twitter',
      source: tweet.author,
      title: this.generateTitle(tweet.content, rule.type),
      description: tweet.content,
      requirements,
      deadline,
      reward,
      priority,
      url: tweet.url,
      timestamp: tweet.timestamp,
      metadata
    };
  }

  private getMaxAgeForContent(content: string): number {
    // Red packets expire quickly
    if (content.includes('红包') || content.includes('red packet')) {
      return 2;
    }
    // Airdrops have longer deadlines
    if (content.includes('airdrop') || content.includes('空投')) {
      return 48;
    }
    // Giveaways vary
    if (content.includes('giveaway') || content.includes('抽奖')) {
      return 72;
    }
    // Default
    return 24;
  }

  private isSuspicious(content: string): boolean {
    return this.suspiciousPatterns.some(pattern => pattern.test(content));
  }

  private findMatchingRule(content: string, author: string): FilterRule | null {
    for (const rule of this.filterRules) {
      // Check if any keyword matches
      const hasKeyword = rule.keywords.some(keyword => content.includes(keyword.toLowerCase()));
      if (!hasKeyword) continue;

      // Check required keywords
      if (rule.requiredKeywords) {
        const hasRequired = rule.requiredKeywords.some(keyword =>
          content.includes(keyword.toLowerCase()) || author.toLowerCase().includes(keyword.toLowerCase())
        );
        if (!hasRequired) continue;
      }

      // Check excluded keywords
      if (rule.excludeKeywords) {
        const hasExcluded = rule.excludeKeywords.some(keyword => content.includes(keyword.toLowerCase()));
        if (hasExcluded) continue;
      }

      return rule;
    }
    return null;
  }

  private calculatePriority(tweet: Tweet, rule: FilterRule): number {
    let priority = rule.priorityBoost;

    // Account tier bonus
    const tier = this.getAccountTier(tweet.author);
    if (tier) {
      priority = Math.min(10, priority * tier.priorityMultiplier);
    }

    // Specific account bonus
    if (rule.accountBoost && rule.accountBoost[tweet.author.toLowerCase()]) {
      priority = Math.min(10, priority + rule.accountBoost[tweet.author.toLowerCase()]);
    }

    // Code presence bonus
    const codes = this.extractCodes(tweet.content);
    if (codes.length > 0) {
      priority = Math.min(10, priority + 1);
    }

    // Urgency bonus
    if (rule.timeConstraints?.urgencyKeywords) {
      const hasUrgency = rule.timeConstraints.urgencyKeywords.some(keyword =>
        tweet.content.toLowerCase().includes(keyword.toLowerCase())
      );
      if (hasUrgency) {
        priority = Math.min(10, priority + 1);
      }
    }

    // Engagement bonus (if available)
    if (tweet.retweetsCount && tweet.likesCount) {
      const engagement = (tweet.retweetsCount || 0) + (tweet.likesCount || 0);
      if (engagement > 1000) priority = Math.min(10, priority + 1);
      if (engagement > 10000) priority = Math.min(10, priority + 1);
    }

    return Math.round(priority);
  }

  private getAccountTier(author: string): AccountTier | null {
    return this.accountTiers.find(tier =>
      tier.accounts.some(account => account.toLowerCase() === author.toLowerCase())
    ) || null;
  }

  private extractMetadata(tweet: Tweet): CryptoOpportunity['metadata'] {
    return {
      author: tweet.author,
      codes: this.extractCodes(tweet.content),
      mentions: this.extractMentions(tweet.content),
      hashtags: this.extractHashtags(tweet.content),
      retweetsCount: tweet.retweetsCount,
      likesCount: tweet.likesCount
    };
  }

  private extractCodes(content: string): string[] {
    const codes: string[] = [];
    this.codePatterns.forEach(pattern => {
      const matches = content.match(pattern);
      if (matches) {
        codes.push(...matches);
      }
    });
    return [...new Set(codes)]; // Remove duplicates
  }

  private extractMentions(content: string): string[] {
    const matches = content.match(/@\w+/g) || [];
    return matches.map(mention => mention.toLowerCase());
  }

  private extractHashtags(content: string): string[] {
    const matches = content.match(/#\w+/g) || [];
    return matches.map(hashtag => hashtag.toLowerCase());
  }

  private extractRequirements(content: string): string[] {
    const requirements: string[] = [];
    const contentLower = content.toLowerCase();

    if (contentLower.includes('follow') || contentLower.includes('关注')) {
      requirements.push('Follow account');
    }
    if (contentLower.includes('retweet') || contentLower.includes('rt') || contentLower.includes('转发')) {
      requirements.push('Retweet');
    }
    if (contentLower.includes('like') || contentLower.includes('点赞')) {
      requirements.push('Like tweet');
    }
    if (contentLower.includes('comment') || contentLower.includes('reply') || contentLower.includes('评论')) {
      requirements.push('Comment on tweet');
    }
    if (contentLower.includes('tag') || contentLower.includes('mention') || contentLower.includes('@')) {
      requirements.push('Tag friends');
    }
    if (this.extractCodes(content).length > 0) {
      requirements.push('Use provided codes');
    }

    return requirements;
  }

  private extractDeadline(content: string): Date | undefined {
    const contentLower = content.toLowerCase();

    // Look for time patterns
    const timePatterns = [
      /(\d{1,2})\s*(hours?|hrs?)/i,
      /(\d{1,2})\s*(days?)/i,
      /(\d{1,2}[\/\-]\d{1,2})/,
      /(expires?|ends?|截止)\s*(\d{1,2}[\/\-]\d{1,2})/i
    ];

    for (const pattern of timePatterns) {
      const match = contentLower.match(pattern);
      if (match) {
        // Simple parsing - could be enhanced
        if (match[0].includes('hour')) {
          const hours = parseInt(match[1]);
          return new Date(Date.now() + hours * 60 * 60 * 1000);
        } else if (match[0].includes('day')) {
          const days = parseInt(match[1]);
          return new Date(Date.now() + days * 24 * 60 * 60 * 1000);
        } else {
          // Default to 24 hours for date patterns
          return new Date(Date.now() + 24 * 60 * 60 * 1000);
        }
      }
    }

    return undefined;
  }

  private extractReward(content: string): string | undefined {
    const rewardPatterns = [
      /(\$?\d+\.?\d*\s*(USDT|BTC|ETH|BNB|USD))/i,
      /(\d+\.?\d*\s*美元)/i,
      /(\d+\.?\d*\s*u)/i,
      /(worth\s*\$?\d+)/i
    ];

    for (const pattern of rewardPatterns) {
      const match = content.match(pattern);
      if (match) {
        return match[1];
      }
    }

    return undefined;
  }

  private generateTitle(content: string, type: CryptoOpportunity['type']): string {
    const maxLength = 100;
    let title = content.substring(0, maxLength);

    if (content.length > maxLength) {
      title += '...';
    }

    // Add type prefix for clarity
    switch (type) {
      case 'red_packet':
        title = `🧧 ${title}`;
        break;
      case 'airdrop':
        title = `🪂 ${title}`;
        break;
      case 'giveaway':
        title = `🎁 ${title}`;
        break;
      case 'alpha':
        title = `💡 ${title}`;
        break;
      default:
        title = `📈 ${title}`;
    }

    return title;
  }

  // Method to get filtering statistics
  getFilteringStats(): any {
    return {
      totalRules: this.filterRules.length,
      accountTiers: this.accountTiers.map(tier => ({
        tier: tier.tier,
        accountCount: tier.accounts.length,
        priorityMultiplier: tier.priorityMultiplier
      })),
      suspiciousPatternsCount: this.suspiciousPatterns.length,
      codePatternCount: this.codePatterns.length
    };
  }
}