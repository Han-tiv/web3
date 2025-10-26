import axios from 'axios';
import { parseStringPromise } from 'xml2js';
import * as cheerio from 'cheerio';
type CheerioRoot = cheerio.Root;

interface Following {
  username: string;
  displayName: string;
  description: string;
  isVerified: boolean;
  profileUrl: string;
  avatarUrl?: string;
}

interface UserFollowingResult {
  username: string;
  totalCount: number;
  following: Following[];
  nextCursor?: string;
}

export interface TimelineTweet {
  id: string;
  content: string;
  timestamp: string;
  author: string;
  url: string;
}

export interface TimelinePage {
  tweets: TimelineTweet[];
  nextCursor?: string;
}

export class NitterUserService {
  private nitterInstances: string[] = [
    process.env.NITTER_URL || 'http://localhost:8080',
    'https://nitter.net',
    'https://nitter.it',
    'https://nitter.unixfox.eu',
    'https://nitter.domain.glass'
  ];

  private async tryNitterInstance(instance: string, endpoint: string): Promise<any> {
    try {
      const response = await axios.get(`${instance}${endpoint}`, {
        timeout: 10000,
        headers: {
          'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        }
      });
      return response;
    } catch (error) {
      console.log(`Failed to connect to ${instance}: ${error}`);
      throw error;
    }
  }

  private async fetchHtml(endpoint: string): Promise<{ instance: string; html: string }> {
    for (const instance of this.nitterInstances) {
      try {
        const response = await this.tryNitterInstance(instance, endpoint);
        return { instance, html: response.data };
      } catch (error) {
        continue;
      }
    }
    throw new Error('All Nitter instances are unavailable');
  }

  private async getWorkingNitterInstance(endpoint: string): Promise<string> {
    for (const instance of this.nitterInstances) {
      try {
        await this.tryNitterInstance(instance, endpoint);
        return instance;
      } catch (error) {
        continue;
      }
    }
    throw new Error('All Nitter instances are unavailable');
  }

  async getUserFollowing(username: string, limit: number = 50): Promise<UserFollowingResult> {
    const cleanUsername = username.replace('@', '');
    const endpoint = `/${cleanUsername}/following`;

    try {
      const { instance: workingInstance, html } = await this.fetchHtml(endpoint);
      console.log(`Using Nitter instance: ${workingInstance}`);

      // Parse HTML with Cheerio
      const $: CheerioRoot = cheerio.load(html);
      const following: Following[] = [];

      // Extract following users from the HTML
      $('.user-card').each((_, element) => {
        if (following.length >= limit) return false;

        const $user = $(element);
        const usernameEl = $user.find('.username');
        const displayNameEl = $user.find('.fullname');
        const descriptionEl = $user.find('.user-bio');
        const avatarEl = $user.find('.avatar img');
        const isVerified = $user.find('.verified-icon').length > 0;

        const userUsername = usernameEl.text().replace('@', '').trim();
        const displayName = displayNameEl.text().trim();
        const description = descriptionEl.text().trim();
        const avatarUrl = avatarEl.attr('src');

        if (userUsername) {
          following.push({
            username: userUsername,
            displayName: displayName || userUsername,
            description: description || '',
            isVerified,
            profileUrl: `https://twitter.com/${userUsername}`,
            avatarUrl: avatarUrl || undefined
          });
        }

        return true; // Continue iteration
      });

      // Try to get total count from the page
      let totalCount = following.length;
      const statsElement = $('.profile-stat-num').first();
      if (statsElement.length > 0) {
        const statsText = statsElement.text().replace(/,/g, '');
        const parsedCount = parseInt(statsText);
        if (!isNaN(parsedCount)) {
          totalCount = parsedCount;
        }
      }

      return {
        username: cleanUsername,
        totalCount,
        following,
        nextCursor: following.length >= limit ? 'has_more' : undefined
      };

    } catch (error) {
      console.error(`Error fetching following for @${cleanUsername}:`, error);

      // Fallback: Return mock data for demonstration
      return {
        username: cleanUsername,
        totalCount: 0,
        following: [],
        nextCursor: undefined
      };
    }
  }

  async getUserProfile(username: string): Promise<any> {
    const cleanUsername = username.replace('@', '');
    const endpoint = `/${cleanUsername}`;

    try {
      const { html } = await this.fetchHtml(endpoint);

      const $: CheerioRoot = cheerio.load(html);

      // Extract profile information
      const displayName = $('.profile-card-fullname').text().trim();
      const bio = $('.profile-bio').text().trim();
      const location = $('.profile-location').text().trim();
      const website = $('.profile-website a').attr('href');
      const joinDate = $('.profile-joindate').text().trim();
      const isVerified = $('.verified-icon').length > 0;

      // Extract stats
      const tweets = $('.profile-stat-num').eq(0).text().replace(/,/g, '');
      const following = $('.profile-stat-num').eq(1).text().replace(/,/g, '');
      const followers = $('.profile-stat-num').eq(2).text().replace(/,/g, '');

      return {
        username: cleanUsername,
        displayName: displayName || cleanUsername,
        bio: bio || '',
        location: location || '',
        website: website || '',
        joinDate: joinDate || '',
        isVerified,
        stats: {
          tweets: parseInt(tweets) || 0,
          following: parseInt(following) || 0,
          followers: parseInt(followers) || 0
        },
        profileUrl: `https://twitter.com/${cleanUsername}`
      };

    } catch (error) {
      console.error(`Error fetching profile for @${cleanUsername}:`, error);
      return null;
    }
  }

  /**
   * 抓取用户时间线的单页数据，可选择传入 cursor。
   */
  async getUserTimelinePage(
    username: string,
    options: { cursor?: string; limit?: number } = {}
  ): Promise<TimelinePage> {
    const cleanUsername = username.replace('@', '');
    const cursor = options.cursor;
    const endpoint = cursor
      ? `/${cleanUsername}?cursor=${encodeURIComponent(cursor)}`
      : `/${cleanUsername}`;

    try {
      const { html } = await this.fetchHtml(endpoint);
      const $: CheerioRoot = cheerio.load(html);
      const tweets = this.extractTimelineTweets($, cleanUsername, options.limit);
      const nextCursor = this.extractNextCursor($);
      return { tweets, nextCursor };
    } catch (error) {
      console.error(`Error fetching timeline page for @${cleanUsername}:`, error);
      return { tweets: [], nextCursor: undefined };
    }
  }

  // Get user's recent tweets (single page, backward compatible)
  async getUserTweets(username: string, limit: number = 20): Promise<TimelineTweet[]> {
    const cleanUsername = username.replace('@', '');
    try {
      const page = await this.getUserTimelinePage(cleanUsername, { limit });
      return page.tweets.slice(0, limit);
    } catch (error) {
      console.error(`Error fetching tweets for @${cleanUsername}:`, error);
      return [];
    }
  }

  private extractTimelineTweets($: CheerioRoot, cleanUsername: string, limit?: number): TimelineTweet[] {
    const tweets: TimelineTweet[] = [];
    $('.timeline-item').each((_, element) => {
      const $tweet = $(element);
      if ($tweet.hasClass('show-more')) {
        return true;
      }

      if (typeof limit === 'number' && tweets.length >= limit) {
        return false;
      }

      const content = $tweet.find('.tweet-content').text().trim();
      const timestamp = $tweet.find('.tweet-date a').attr('title');
      const tweetHref = $tweet.find('.tweet-link').attr('href');
      const tweetId = tweetHref?.split('/').pop()?.split('?')[0];

      if (content && tweetId) {
        tweets.push({
          id: tweetId,
          content,
          timestamp: timestamp || new Date().toISOString(),
          author: cleanUsername,
          url: `https://twitter.com/${cleanUsername}/status/${tweetId}`
        });
      }

      return true;
    });
    return tweets;
  }

  private extractNextCursor($: CheerioRoot): string | undefined {
    const anchor = $('.show-more a[href*="cursor="]').last();
    const href = anchor.attr('href');
    if (!href) {
      return undefined;
    }
    const match = href.match(/cursor=([^&]+)/);
    if (!match) {
      return undefined;
    }
    return match[1];
  }
}
