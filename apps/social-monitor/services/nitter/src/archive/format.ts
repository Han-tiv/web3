import { AccountArchive, ArchivedTweet, FollowingAccount } from './types';

const INVALID_FILENAME = /[^a-zA-Z0-9._-]/g;

/**
 * 将账号标识转换为安全的目录名，过滤非法字符并避免结果为空。
 */
export function sanitizeHandle(handle: string): string {
  const normalized = handle.trim().toLowerCase();
  const replaced = normalized.replace(/\s+/g, '-').replace(INVALID_FILENAME, '_');
  return replaced.length > 0 ? replaced : 'unknown-account';
}

/**
 * 将抓取结果组装为便于 LLM 阅读的 Markdown 文本。
 */
export function buildMarkdown(archive: AccountArchive): string {
  const { account, tweets, meta } = archive;
  const lines: string[] = [];

  lines.push(`# ${getAccountTitle(account)}`);
  lines.push('');
  lines.push(`- 抓取时间：${meta.capturedAt}`);
  lines.push(`- 推文数量：${tweets.length} / 目标 ${meta.tweetLimit}`);
  if (typeof meta.durationMs === 'number') {
    lines.push(`- 抓取耗时：${Math.round(meta.durationMs)} ms`);
  }
  lines.push(`- 数据来源：${meta.sourceFile}`);
  lines.push(`- 分页次数：${meta.pagesFetched}`);
  lines.push(`- 抓取尝试：${meta.attempts}`);
  if (meta.cursorTrail?.length) {
    lines.push(`- Cursor 链：${meta.cursorTrail.length} 条`);
  }
  if (meta.skipped) {
    lines.push('- 状态：已有归档文件，本次跳过抓取');
  }

  appendAccountStats(lines, account);

  if (meta.errors?.length) {
    lines.push(`- ⚠️ 抓取异常：${meta.errors.join('；')}`);
  }

  if (account.description) {
    lines.push('');
    lines.push('> 账号简介：');
    account.description.split('\n').forEach(line => {
      lines.push(`> ${line.trim() || ' '}`);
    });
  }

  lines.push('');
  lines.push('---');
  lines.push('');

  if (tweets.length === 0) {
    lines.push('（暂无可用推文）');
    return lines.join('\n');
  }

  tweets.forEach((tweet, index) => {
    lines.push(`## ${index + 1}. ${formatDate(tweet.timestamp)} · ${tweet.id}`);
    lines.push(`- 链接：${tweet.url}`);
    lines.push(`- 作者：@${tweet.author}`);
    lines.push('');
    tweet.content.split('\n').forEach(line => {
      lines.push(`> ${line.trim() || ' '}`);
    });
    lines.push('');
  });

  return lines.join('\n');
}

function appendAccountStats(lines: string[], account: FollowingAccount): void {
  const stats: string[] = [];

  if (typeof account.followers_count === 'number') {
    stats.push(`粉丝 ${account.followers_count}`);
  }
  if (typeof account.friends_count === 'number') {
    stats.push(`关注 ${account.friends_count}`);
  }
  if (typeof account.statuses_count === 'number') {
    stats.push(`推文 ${account.statuses_count}`);
  }
  if (typeof account.favourites_count === 'number') {
    stats.push(`点赞 ${account.favourites_count}`);
  }

  if (stats.length > 0) {
    lines.push(`- 账号统计：${stats.join('，')}`);
  }

  if (account.location) {
    lines.push(`- 所在地：${account.location}`);
  }
  if (account.website) {
    lines.push(`- 站外链接：${account.website}`);
  }
  if (account.url) {
    lines.push(`- Twitter：${account.url}`);
  }
  if (account.profile_image_url) {
    lines.push(`- 头像：${account.profile_image_url}`);
  }
  if (account.profile_banner_url) {
    lines.push(`- Banner：${account.profile_banner_url}`);
  }
  if (account.is_blue_verified || account.verified_type) {
    lines.push(`- 认证：${account.verified_type || 'Blue Verified'}`);
  }
  if (account.protected) {
    lines.push('- 隐私状态：受保护账号');
  }
}

function getAccountTitle(account: FollowingAccount): string {
  const displayName = account.name || account.screen_name;
  return `${displayName} (@${account.screen_name})`;
}

function formatDate(isoString: string): string {
  const parsed = new Date(isoString);
  if (Number.isNaN(parsed.getTime())) {
    return isoString;
  }
  return parsed.toISOString().replace('T', ' ').replace('Z', ' UTC');
}

/**
 * 将基础推文数组按 ID 去重后返回，防止重复写入。
 */
export function deduplicateTweets(tweets: ArchivedTweet[]): ArchivedTweet[] {
  const seen = new Map<string, ArchivedTweet>();
  tweets.forEach(tweet => {
    if (!seen.has(tweet.id)) {
      seen.set(tweet.id, tweet);
    }
  });
  return Array.from(seen.values());
}
