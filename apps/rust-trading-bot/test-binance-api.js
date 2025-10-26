#!/usr/bin/env node

// 快速测试Binance API连接
const axios = require('axios');
const crypto = require('crypto');

// 从环境变量读取API密钥
const API_KEY = process.env.BINANCE_API_KEY || 'your_api_key_here';
const SECRET_KEY = process.env.BINANCE_SECRET_KEY || 'your_secret_key_here';
const TESTNET = process.env.BINANCE_TESTNET === 'true';

// API地址
const BASE_URL = TESTNET
  ? 'https://testnet.binancefuture.com'  // 测试网
  : 'https://fapi.binance.com';          // 主网

// 创建签名
function createSignature(queryString, secret) {
  return crypto
    .createHmac('sha256', secret)
    .update(queryString)
    .digest('hex');
}

// 测试函数们
async function testConnection() {
  console.log('🔍 测试Binance连接...');
  try {
    const response = await axios.get(`${BASE_URL}/fapi/v1/ping`);
    console.log('✅ Binance连接正常');
    return true;
  } catch (error) {
    console.log('❌ Binance连接失败:', error.message);
    return false;
  }
}

async function testMarketData() {
  console.log('\n📊 测试市场数据获取...');
  try {
    const response = await axios.get(`${BASE_URL}/fapi/v1/ticker/24hr?symbol=BTCUSDT`);
    const data = response.data;
    console.log('✅ 市场数据获取成功:');
    console.log(`   BTC/USDT 价格: $${parseFloat(data.lastPrice).toLocaleString()}`);
    console.log(`   24h变化: ${data.priceChangePercent}%`);
    console.log(`   24h成交量: ${parseFloat(data.volume).toLocaleString()} BTC`);
    return true;
  } catch (error) {
    console.log('❌ 市场数据获取失败:', error.message);
    return false;
  }
}

async function testAccountInfo() {
  console.log('\n👤 测试账户信息获取...');

  if (API_KEY === 'your_api_key_here' || SECRET_KEY === 'your_secret_key_here') {
    console.log('⚠️  请先设置API密钥');
    console.log('   export BINANCE_API_KEY=你的API密钥');
    console.log('   export BINANCE_SECRET_KEY=你的SECRET密钥');
    console.log('   export BINANCE_TESTNET=true  # 如果使用测试网');
    return false;
  }

  try {
    const timestamp = Date.now();
    const queryString = `timestamp=${timestamp}`;
    const signature = createSignature(queryString, SECRET_KEY);

    const response = await axios.get(`${BASE_URL}/fapi/v2/account?${queryString}&signature=${signature}`, {
      headers: {
        'X-MBX-APIKEY': API_KEY
      }
    });

    const account = response.data;
    console.log('✅ 账户信息获取成功:');
    console.log(`   账户余额: ${account.totalWalletBalance} USDT`);
    console.log(`   可用余额: ${account.availableBalance} USDT`);
    console.log(`   持仓数量: ${account.positions.filter(p => parseFloat(p.positionAmt) !== 0).length}`);

    // 显示非零持仓
    const activePositions = account.positions.filter(p => parseFloat(p.positionAmt) !== 0);
    if (activePositions.length > 0) {
      console.log('\n📦 当前持仓:');
      activePositions.forEach(pos => {
        const side = parseFloat(pos.positionAmt) > 0 ? 'LONG' : 'SHORT';
        const size = Math.abs(parseFloat(pos.positionAmt));
        const pnl = parseFloat(pos.unrealizedProfit);
        const pnlEmoji = pnl > 0 ? '🟢' : '🔴';
        console.log(`   ${pos.symbol}: ${side} ${size} (PnL: ${pnl.toFixed(2)} USDT) ${pnlEmoji}`);
      });
    }

    return true;
  } catch (error) {
    console.log('❌ 账户信息获取失败:', error.response?.data?.msg || error.message);
    if (error.response?.status === 401) {
      console.log('💡 可能原因:');
      console.log('   1. API Key错误');
      console.log('   2. Secret Key错误');
      console.log('   3. API权限不足（需要期货交易权限）');
      console.log('   4. IP白名单限制');
    }
    return false;
  }
}

async function testOrderHistory() {
  console.log('\n📋 测试订单历史获取...');

  if (API_KEY === 'your_api_key_here') {
    console.log('⚠️  需要API密钥才能获取订单历史');
    return false;
  }

  try {
    const timestamp = Date.now();
    const queryString = `symbol=BTCUSDT&limit=5&timestamp=${timestamp}`;
    const signature = createSignature(queryString, SECRET_KEY);

    const response = await axios.get(`${BASE_URL}/fapi/v1/allOrders?${queryString}&signature=${signature}`, {
      headers: {
        'X-MBX-APIKEY': API_KEY
      }
    });

    const orders = response.data;
    console.log(`✅ 获取到 ${orders.length} 条订单历史`);

    if (orders.length > 0) {
      console.log('   最近订单:');
      orders.slice(0, 3).forEach(order => {
        const time = new Date(order.time).toLocaleString('zh-CN');
        console.log(`   ${order.symbol} ${order.side} ${order.origQty} @ ${order.price} (${time})`);
      });
    }

    return true;
  } catch (error) {
    console.log('❌ 订单历史获取失败:', error.response?.data?.msg || error.message);
    return false;
  }
}

// 主测试函数
async function runTests() {
  console.log('🚀 Binance API 测试开始');
  console.log(`📡 使用${TESTNET ? '测试网' : '主网'}: ${BASE_URL}\n`);

  const results = [];

  // 基础连接测试
  results.push(await testConnection());

  // 市场数据测试（无需API密钥）
  results.push(await testMarketData());

  // 账户信息测试（需要API密钥）
  results.push(await testAccountInfo());

  // 订单历史测试（需要API密钥）
  results.push(await testOrderHistory());

  // 测试结果汇总
  const passed = results.filter(r => r).length;
  const total = results.length;

  console.log('\n' + '='.repeat(50));
  console.log(`📊 测试结果: ${passed}/${total} 通过`);

  if (passed === total) {
    console.log('🎉 所有测试通过！Binance API配置正确');
  } else if (passed >= 2) {
    console.log('⚠️  基础功能正常，请检查API密钥配置');
  } else {
    console.log('❌ 多项测试失败，请检查网络和配置');
  }

  console.log('\n💡 提示:');
  console.log('   - 测试网和主网需要不同的API密钥');
  console.log('   - 确保API权限包含"期货交易"');
  console.log('   - 检查IP白名单设置');

  return passed === total;
}

// 运行测试
if (require.main === module) {
  runTests().catch(console.error);
}

module.exports = { runTests };