require('ts-node/register');

const assert = require('assert');
const Winston = require('winston');

const { toBoolean } = require('../src/clients/valueScanClient');
const { buildTelegramMessage, ValueScanWatcher } = require('../src/watchers/valueScanWatcher');

class RedisMock {
  constructor() {
    this.store = new Map();
  }

  async sIsMember(key, value) {
    const set = this.store.get(key);
    return set && set.has(value) ? 1 : 0;
  }

  async sAdd(key, value) {
    const set = this.store.get(key) || new Set();
    const before = set.size;
    set.add(value);
    this.store.set(key, set);
    return set.size > before ? 1 : 0;
  }

  async expire() {
    return 1;
  }
}

class DummyClient {
  constructor(list) {
    this.list = list;
  }

  async fetchFundsMovement() {
    return this.list;
  }
}

class DummyNotifier {
  constructor() {
    this.messages = [];
  }

  async sendMessage(text) {
    this.messages.push(text);
  }
}

const loggerStub = Winston.createLogger({
  level: 'error',
  transports: [new Winston.transports.Console({ silent: true })]
});

(async () => {
  try {
    assert.strictEqual(toBoolean(true), true);
    assert.strictEqual(toBoolean('alpha'), true);
    assert.strictEqual(toBoolean('1'), true);
    assert.strictEqual(toBoolean('false'), false);
    assert.strictEqual(toBoolean(''), false);

    const now = Date.now();
    const sampleItem = {
      id: 'test-1',
      updateTime: now,
      tradeType: 2,
      symbol: 'XPIN',
      beginTime: now - 60_000,
      endTime: now,
      number24h: 18,
      numberNot24h: 1,
      price: 0.0034284,
      beginPrice: 0.0024193,
      gains: 66.45,
      decline: 2.45,
      percentChange24h: 83.84,
      marketCap: 61671359.91,
      alpha: true,
      fomo: false,
      icon: null
    };

    const message = buildTelegramMessage(sampleItem);
    assert(message.includes('XPIN'), '消息应包含代币符号');
    assert(message.toLowerCase().includes('alpha'), '消息应包含标签 alpha');

    const redis = new RedisMock();
    const client = new DummyClient([sampleItem]);
    const notifier = new DummyNotifier();
    const watcher = new ValueScanWatcher(redis, client, notifier, loggerStub);

    await watcher.run();
    assert.strictEqual(notifier.messages.length, 1, '首次运行应发送一次通知');

    await watcher.run();
    assert.strictEqual(notifier.messages.length, 1, '重复运行应触发去重');

    console.log('ValueScanWatcher tests passed');
  } catch (error) {
    console.error('ValueScanWatcher tests failed');
    console.error(error);
    process.exit(1);
  }
})();
