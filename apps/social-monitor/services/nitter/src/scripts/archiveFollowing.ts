import path from 'path';
import fs from 'fs/promises';
import { setTimeout as delay } from 'timers/promises';
import {
  NitterUserService,
  TimelineTweet
} from '../userService';
import {
  buildMarkdown,
  deduplicateTweets,
  sanitizeHandle
} from '../archive/format';
import {
  AccountArchive,
  ArchivedTweet,
  ArchiveMeta,
  FollowingAccount
} from '../archive/types';

interface CliOptions {
  inputPath: string;
  outputDir: string;
  tweetLimit: number;
  delayMs: number;
  pageDelayMs: number;
  retries: number;
  maxPages?: number;
  limitAccounts?: number;
  handlesFilter?: Set<string>;
  skipMarkdown: boolean;
  skipExisting: boolean;
  dryRun: boolean;
  stateFile: string;
}

interface FetchResult {
  tweets: ArchivedTweet[];
  pagesFetched: number;
  cursorTrail: string[];
  errors: string[];
  durationMs: number;
  attempts: number;
}

interface ArchiveRunResult {
  handle: string;
  tweetsFetched: number;
  pagesFetched: number;
  durationMs: number;
  attempts: number;
  errors?: string[];
  outputDir: string;
  skipped: boolean;
}

interface ArchiveRunSummary {
  startedAt: string;
  finishedAt?: string;
  options: {
    tweetLimit: number;
    delayMs: number;
    pageDelayMs: number;
    retries: number;
    maxPages?: number;
    handlesPlanned: number;
  };
  results: ArchiveRunResult[];
}

/**
 * 基于命令行参数配置抓取行为。
 */
function parseArgs(argv: string[]): CliOptions {
  const defaults = {
    inputPath: path.resolve(process.cwd(), '../../../..', 'twitter-Following-1760964620895.json'),
    outputDir: path.resolve(process.cwd(), 'data', 'following'),
    tweetLimit: 200,
    delayMs: 3000,
    pageDelayMs: 1500,
    retries: 2,
    maxPages: 50,
    limitAccounts: undefined,
    handlesFilter: undefined,
    skipMarkdown: false,
    skipExisting: false,
    dryRun: false,
    stateFile: undefined as string | undefined
  };

  const options: Partial<CliOptions> = {};

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith('--')) continue;

    const [flag, inlineValue] = arg.includes('=') ? arg.split('=', 2) : [arg, undefined];
    const value = inlineValue ?? argv[i + 1];

    switch (flag) {
      case '--input':
        if (!value) throw new Error('缺少 --input 参数值');
        options.inputPath = path.resolve(process.cwd(), value);
        if (!inlineValue) i++;
        break;
      case '--output':
        if (!value) throw new Error('缺少 --output 参数值');
        options.outputDir = path.resolve(process.cwd(), value);
        if (!inlineValue) i++;
        break;
      case '--tweet-limit':
        if (!value) throw new Error('缺少 --tweet-limit 参数值');
        options.tweetLimit = toPositiveInt(value, 'tweet-limit');
        if (!inlineValue) i++;
        break;
      case '--delay':
        if (!value) throw new Error('缺少 --delay 参数值');
        options.delayMs = toNonNegativeInt(value, 'delay');
        if (!inlineValue) i++;
        break;
      case '--page-delay':
        if (!value) throw new Error('缺少 --page-delay 参数值');
        options.pageDelayMs = toNonNegativeInt(value, 'page-delay');
        if (!inlineValue) i++;
        break;
      case '--retries':
        if (!value) throw new Error('缺少 --retries 参数值');
        options.retries = toNonNegativeInt(value, 'retries');
        if (!inlineValue) i++;
        break;
      case '--max-pages':
        if (!value) throw new Error('缺少 --max-pages 参数值');
        options.maxPages = toNonNegativeInt(value, 'max-pages');
        if (!inlineValue) i++;
        break;
      case '--limit-accounts':
        if (!value) throw new Error('缺少 --limit-accounts 参数值');
        options.limitAccounts = toPositiveInt(value, 'limit-accounts');
        if (!inlineValue) i++;
        break;
      case '--handles':
        if (!value) throw new Error('缺少 --handles 参数值');
        options.handlesFilter = new Set(
          value.split(',').map(item => item.trim().toLowerCase()).filter(Boolean)
        );
        if (!inlineValue) i++;
        break;
      case '--skip-markdown':
        options.skipMarkdown = true;
        break;
      case '--skip-existing':
        options.skipExisting = true;
        break;
      case '--dry-run':
        options.dryRun = true;
        break;
      case '--state-file':
        if (!value) throw new Error('缺少 --state-file 参数值');
        options.stateFile = path.resolve(process.cwd(), value);
        if (!inlineValue) i++;
        break;
      default:
        throw new Error(`未知参数：${flag}`);
    }
  }

  const merged: CliOptions = {
    ...defaults,
    ...options,
    // stateFile 默认放在输出目录
    stateFile: options.stateFile
      ? options.stateFile
      : path.join(options.outputDir ?? defaults.outputDir, 'archive-state.json')
  } as CliOptions;

  return merged;
}

function toPositiveInt(value: string, field: string): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${field} 必须是正整数`);
  }
  return parsed;
}

function toNonNegativeInt(value: string, field: string): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error(`${field} 必须是非负整数`);
  }
  return parsed;
}

async function ensureDir(dir: string): Promise<void> {
  await fs.mkdir(dir, { recursive: true });
}

async function pathExists(filePath: string): Promise<boolean> {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function loadFollowing(inputPath: string): Promise<FollowingAccount[]> {
  const raw = await fs.readFile(inputPath, 'utf8');
  const parsed = JSON.parse(raw);
  if (!Array.isArray(parsed)) {
    throw new Error('关注列表文件格式必须为数组');
  }
  return parsed as FollowingAccount[];
}

function filterAccounts(
  accounts: FollowingAccount[],
  { handlesFilter, limitAccounts }: Pick<CliOptions, 'handlesFilter' | 'limitAccounts'>
): FollowingAccount[] {
  let result = accounts;

  if (handlesFilter && handlesFilter.size > 0) {
    result = result.filter(account => handlesFilter.has(account.screen_name.toLowerCase()));
  }

  if (typeof limitAccounts === 'number') {
    result = result.slice(0, limitAccounts);
  }

  return result;
}

function normalizeTweet(raw: TimelineTweet, handle: string): ArchivedTweet {
  const id = raw.id ? String(raw.id) : '';
  const timestamp = safeTimestamp(raw.timestamp);
  const content = typeof raw.content === 'string' ? raw.content.trim() : '';
  const url =
    typeof raw.url === 'string' && raw.url.length > 0
      ? raw.url
      : `https://twitter.com/${handle}/status/${id}`;

  return {
    id,
    content,
    timestamp,
    url,
    author: raw.author || handle
  };
}

function safeTimestamp(value?: string): string {
  if (!value) {
    return new Date().toISOString();
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return parsed.toISOString();
}

async function collectTweetsWithRetry(
  userService: NitterUserService,
  handle: string,
  limit: number,
  maxPages: number | undefined,
  pageDelayMs: number,
  retries: number
): Promise<FetchResult> {
  const errors: string[] = [];
  const started = Date.now();
  let attempts = 0;
  let lastResult: { tweets: ArchivedTweet[]; pagesFetched: number; cursorTrail: string[] } = {
    tweets: [],
    pagesFetched: 0,
    cursorTrail: []
  };

  while (attempts <= retries) {
    attempts += 1;
    try {
      lastResult = await collectTweets(
        userService,
        handle,
        limit,
        maxPages,
        pageDelayMs
      );

      if (lastResult.tweets.length > 0 || attempts > retries) {
        break;
      }
      errors.push(`第 ${attempts} 次抓取结果为空，准备重试`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      errors.push(`第 ${attempts} 次抓取异常：${message}`);
    }

    if (attempts <= retries) {
      await delay(1000 * attempts);
    }
  }

  return {
    tweets: deduplicateTweets(lastResult.tweets),
    pagesFetched: lastResult.pagesFetched,
    cursorTrail: lastResult.cursorTrail,
    errors,
    durationMs: Date.now() - started,
    attempts
  };
}

async function collectTweets(
  userService: NitterUserService,
  handle: string,
  limit: number,
  maxPages: number | undefined,
  pageDelayMs: number
): Promise<{ tweets: ArchivedTweet[]; pagesFetched: number; cursorTrail: string[] }> {
  const effectiveLimit = limit > 0 ? limit : Number.POSITIVE_INFINITY;
  const effectiveMaxPages = typeof maxPages === 'number' && maxPages > 0
    ? maxPages
    : Number.POSITIVE_INFINITY;

  const tweets: ArchivedTweet[] = [];
  const cursorTrail: string[] = [];
  const seen = new Set<string>();
  let cursor: string | undefined;
  let pagesFetched = 0;

  while (pagesFetched < effectiveMaxPages) {
    const page = await userService.getUserTimelinePage(handle, { cursor });
    pagesFetched += 1;

    page.tweets.forEach(rawTweet => {
      if (tweets.length >= effectiveLimit) {
        return;
      }
      if (!rawTweet.id) {
        return;
      }
      if (seen.has(rawTweet.id)) {
        return;
      }
      seen.add(rawTweet.id);
      tweets.push(normalizeTweet(rawTweet, handle));
    });

    if (!page.nextCursor) {
      break;
    }

    if (tweets.length >= effectiveLimit) {
      break;
    }

    if (cursorTrail.includes(page.nextCursor)) {
      break;
    }

    cursorTrail.push(page.nextCursor);
    cursor = page.nextCursor;

    if (pageDelayMs > 0) {
      const jitter = Math.floor(Math.random() * Math.max(500, Math.round(pageDelayMs * 0.2)));
      await delay(pageDelayMs + jitter);
    }
  }

  return {
    tweets: effectiveLimit === Number.POSITIVE_INFINITY ? tweets : tweets.slice(0, effectiveLimit),
    pagesFetched,
    cursorTrail
  };
}

function toArchive(account: FollowingAccount, meta: ArchiveMeta, tweets: ArchivedTweet[]): AccountArchive {
  const { metadata, ...rest } = account as FollowingAccount & { metadata?: any };

  const snapshot: FollowingAccount = {
    ...rest,
    description: account.description ?? '',
    website: account.website ?? ''
  };

  if (metadata?.rest_id) {
    snapshot.rest_id = metadata.rest_id;
  }
  if (metadata?.core?.created_at && !snapshot.created_at) {
    snapshot.created_at = metadata.core.created_at;
  }

  return {
    account: snapshot,
    tweets,
    meta
  };
}

async function writeArchiveFiles(
  archive: AccountArchive,
  accountDir: string,
  skipMarkdown: boolean,
  dryRun: boolean
): Promise<void> {
  if (dryRun) {
    console.log(`[dry-run] 将在 ${accountDir} 写入 ${archive.tweets.length} 条推文`);
    return;
  }

  await ensureDir(accountDir);

  const jsonPath = path.join(accountDir, 'tweets.json');
  await fs.writeFile(jsonPath, JSON.stringify(archive, null, 2), 'utf8');

  if (!skipMarkdown) {
    const markdownPath = path.join(accountDir, 'tweets.md');
    const markdown = buildMarkdown(archive);
    await fs.writeFile(markdownPath, markdown, 'utf8');
  }
}

async function persistSummary(
  summary: ArchiveRunSummary,
  stateFile: string,
  dryRun: boolean
): Promise<void> {
  if (dryRun) {
    return;
  }
  await ensureDir(path.dirname(stateFile));
  summary.finishedAt = new Date().toISOString();
  await fs.writeFile(stateFile, JSON.stringify(summary, null, 2), 'utf8');
}

async function main(): Promise<void> {
  const options = parseArgs(process.argv.slice(2));
  console.log('📥 归档配置：', {
    input: options.inputPath,
    output: options.outputDir,
    tweetLimit: options.tweetLimit,
    delayMs: options.delayMs,
    pageDelayMs: options.pageDelayMs,
    retries: options.retries,
    maxPages: options.maxPages,
    limitAccounts: options.limitAccounts,
    handlesFilter: options.handlesFilter ? Array.from(options.handlesFilter) : undefined,
    skipMarkdown: options.skipMarkdown,
    skipExisting: options.skipExisting,
    dryRun: options.dryRun,
    stateFile: options.stateFile
  });

  const accounts = await loadFollowing(options.inputPath);
  const filtered = filterAccounts(accounts, options);
  console.log(`📚 输入账号总数：${accounts.length}，本次处理：${filtered.length}`);

  if (!options.dryRun) {
    await ensureDir(options.outputDir);
  }

  const summary: ArchiveRunSummary = {
    startedAt: new Date().toISOString(),
    options: {
      tweetLimit: options.tweetLimit,
      delayMs: options.delayMs,
      pageDelayMs: options.pageDelayMs,
      retries: options.retries,
      maxPages: options.maxPages,
      handlesPlanned: filtered.length
    },
    results: []
  };

  const userService = new NitterUserService();

  for (let index = 0; index < filtered.length; index++) {
    const account = filtered[index];
    const handle = account.screen_name;
    const safeHandle = sanitizeHandle(handle);
    const accountDir = path.join(options.outputDir, safeHandle);
    const jsonPath = path.join(accountDir, 'tweets.json');

    console.log(`\n[${index + 1}/${filtered.length}] ⏳ 处理 @${handle} ...`);

    if (options.skipExisting && !options.dryRun && await pathExists(jsonPath)) {
      console.log(`⚠️ 检测到已存在的归档文件 ${jsonPath}，根据参数跳过抓取。`);
      summary.results.push({
        handle,
        tweetsFetched: 0,
        pagesFetched: 0,
        durationMs: 0,
        attempts: 0,
        errors: undefined,
        outputDir: accountDir,
        skipped: true
      });
      await persistSummary(summary, options.stateFile, options.dryRun);
      continue;
    }

    const fetchResult = await collectTweetsWithRetry(
      userService,
      handle,
      options.tweetLimit,
      options.maxPages,
      options.pageDelayMs,
      options.retries
    );

    const archiveMeta: ArchiveMeta = {
      capturedAt: new Date().toISOString(),
      tweetLimit: options.tweetLimit,
      tweetsFetched: fetchResult.tweets.length,
      pagesFetched: fetchResult.pagesFetched,
      sourceFile: path.basename(options.inputPath),
      errors: fetchResult.errors.length ? fetchResult.errors : undefined,
      durationMs: fetchResult.durationMs,
      cursorTrail: fetchResult.cursorTrail.length ? fetchResult.cursorTrail : undefined,
      attempts: fetchResult.attempts,
      skipped: false
    };

    const archive = toArchive(account, archiveMeta, fetchResult.tweets);

    await writeArchiveFiles(
      archive,
      accountDir,
      options.skipMarkdown,
      options.dryRun
    );

    console.log(
      `✅ 完成 @${handle}，写入 ${fetchResult.tweets.length} 条推文（分页 ${fetchResult.pagesFetched} 次，尝试 ${fetchResult.attempts} 次，耗时 ${Math.round(fetchResult.durationMs)} ms）`
    );
    if (fetchResult.errors.length) {
      fetchResult.errors.forEach(msg => console.warn(`⚠️ ${msg}`));
    }

    summary.results.push({
      handle,
      tweetsFetched: fetchResult.tweets.length,
      pagesFetched: fetchResult.pagesFetched,
      durationMs: fetchResult.durationMs,
      attempts: fetchResult.attempts,
      errors: fetchResult.errors.length ? fetchResult.errors : undefined,
      outputDir: accountDir,
      skipped: false
    });
    await persistSummary(summary, options.stateFile, options.dryRun);

    if (index < filtered.length - 1 && options.delayMs > 0) {
      const jitter = Math.floor(Math.random() * Math.max(1000, Math.round(options.delayMs * 0.3)));
      await delay(options.delayMs + jitter);
    }
  }

  console.log('\n🎉 处理完成');
}

if (require.main === module) {
  main().catch(error => {
    console.error('归档过程中出现错误：', error);
    process.exitCode = 1;
  });
}

export {
  buildMarkdown,
  deduplicateTweets,
  sanitizeHandle,
  parseArgs,
  loadFollowing,
  filterAccounts,
  collectTweetsWithRetry as fetchTweetsWithRetry,
  normalizeTweet,
  toArchive
};
