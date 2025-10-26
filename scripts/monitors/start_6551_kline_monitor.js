#!/usr/bin/env node

/**
 * 6551.io K线监控服务启动脚本
 * 调整为处理K线数据而不是Twitter数据
 */

require('dotenv').config();
const WebSocket = require('ws');

async function start6551Monitor() {
  console.log('🚀 启动6551.io K线监控服务...');

  // 检查必需的环境变量
  if (!process.env.TWITTER_TOKEN) {
    console.error('❌ TWITTER_TOKEN未设置，请在.env文件中配置');
    process.exit(1);
  }

  const config = {
    wsUrl: process.env.TWITTER_WS_URL || 'wss://api.6551.io/kline/ws',
    token: process.env.TWITTER_TOKEN,
  };

  console.log('📊 配置信息:');
  console.log(`   WebSocket URL: ${config.wsUrl}`);
  console.log(`   Token: ${config.token.substring(0, 20)}...`);

  try {
    // 建立WebSocket连接
    const wsUrl = `${config.wsUrl}?token=${config.token}`;
    const ws = new WebSocket(wsUrl);

    ws.on('open', () => {
      console.log('✅ 已连接到6551.io K线WebSocket');

      // 发送心跳或订阅消息
      ws.send(JSON.stringify({
        method: 'SUBSCRIBE',
        params: ['btcusdt@kline_1m'],
        id: 1
      }));
    });

    ws.on('message', (data) => {
      try {
        const message = JSON.parse(data.toString());
        console.log('📊 收到K线数据:');
        console.log(JSON.stringify(message, null, 2));

        // 这里可以添加K线分析逻辑
        if (message.k) {
          const kline = message.k;
          console.log(`💰 ${kline.s}: 开盘${kline.o} 收盘${kline.c} 成交量${kline.v}`);
        }

      } catch (error) {
        console.log('📨 收到原始数据:', data.toString());
      }
    });

    ws.on('error', (error) => {
      console.error('❌ WebSocket错误:', error.message);
    });

    ws.on('close', (code, reason) => {
      console.log(`⚠️  连接关闭: ${code} - ${reason}`);

      // 尝试重连
      setTimeout(() => {
        console.log('🔄 尝试重新连接...');
        start6551Monitor();
      }, 5000);
    });

    // 心跳机制
    const heartbeat = setInterval(() => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.ping();
      }
    }, 30000);

    // 优雅关闭
    process.on('SIGINT', () => {
      console.log('\\n⏹️ 正在停止6551.io监控...');
      clearInterval(heartbeat);
      ws.close();
      process.exit(0);
    });

  } catch (error) {
    console.error('💥 启动失败:', error.message);
    process.exit(1);
  }
}

// 启动服务
start6551Monitor().catch(error => {
  console.error('💥 启动失败:', error);
  process.exit(1);
});