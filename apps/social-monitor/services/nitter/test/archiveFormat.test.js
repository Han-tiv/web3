const assert = require('assert');

const {
  sanitizeHandle,
  buildMarkdown,
  deduplicateTweets
} = require('../dist/archive/format');

const {
  filterAccounts,
  toArchive
} = require('../dist/scripts/archiveFollowing');

function testSanitizeHandle() {
  console.log('🔧 Testing sanitizeHandle...');
  assert.strictEqual(sanitizeHandle('Binance '), 'binance');
  assert.strictEqual(sanitizeHandle('Some User!'), 'some-user_');
  assert.strictEqual(sanitizeHandle(''), 'unknown-account');
}

function testBuildMarkdown() {
  console.log('📝 Testing buildMarkdown...');
  const archive = toArchive(
    {
      id: '1',
      screen_name: 'testuser',
      name: 'Test User',
      description: 'Line1\nLine2',
      followers_count: 10,
      friends_count: 5,
      statuses_count: 3,
      favourites_count: 1,
      location: 'Internet',
      website: 'https://example.com',
      url: 'https://twitter.com/testuser',
      profile_image_url: 'https://example.com/avatar.png',
      profile_banner_url: 'https://example.com/banner.png',
      is_blue_verified: true
    },
    {
      capturedAt: '2025-10-20T00:00:00.000Z',
      tweetLimit: 2,
      tweetsFetched: 1,
      pagesFetched: 2,
      sourceFile: 'twitter-following.json',
      durationMs: 1234,
      cursorTrail: ['CURSOR_A'],
      attempts: 1,
      skipped: false
    },
    deduplicateTweets([
      {
        id: '123',
        content: 'Hello\nWorld',
        timestamp: '2025-10-19T00:00:00.000Z',
        url: 'https://twitter.com/testuser/status/123',
        author: 'testuser'
      }
    ])
  );

  const markdown = buildMarkdown(archive);
  assert.ok(markdown.includes('# Test User (@testuser)'));
  assert.ok(markdown.includes('粉丝 10'));
  assert.ok(markdown.includes('分页次数：2'));
  assert.ok(markdown.includes('抓取尝试：1'));
  assert.ok(markdown.includes('Hello'));
  assert.ok(markdown.includes('World'));
}

function testFilterAccounts() {
  console.log('🔍 Testing filterAccounts...');
  const accounts = [
    { screen_name: 'A', id: '1', name: 'A' },
    { screen_name: 'B', id: '2', name: 'B' },
    { screen_name: 'C', id: '3', name: 'C' }
  ];

  const filtered = filterAccounts(accounts, {
    handlesFilter: new Set(['b']),
    limitAccounts: 1
  });

  assert.strictEqual(filtered.length, 1);
  assert.strictEqual(filtered[0].screen_name, 'B');
}

function run() {
  testSanitizeHandle();
  testBuildMarkdown();
  testFilterAccounts();
  console.log('✅ archive format tests completed');
}

run();
