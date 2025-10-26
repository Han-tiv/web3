#!/usr/bin/env node

/**
 * 6551.io Twitter监控服务启动脚本
 */

require('dotenv').config();
const { TwitterMonitorService } = require('./apps/kronos-defi/packages/twitter-monitor/dist/twitter-monitor.js');

async function startTwitterMonitor() {
  console.log('🚀 启动6551.io Twitter监控服务...');

  // 检查必需的环境变量
  if (!process.env.TWITTER_TOKEN) {
    console.error('❌ TWITTER_TOKEN未设置，请在.env文件中配置');
    console.log('💡 需要从 https://6551.io 获取API Token');
    process.exit(1);
  }

  // 配置监控服务
  const config = {
    wsUrl: process.env.TWITTER_WS_URL || 'wss://ai.6551.io/ws',
    token: process.env.TWITTER_TOKEN,
    enableSentimentAnalysis: process.env.TWITTER_SENTIMENT_ENABLED === 'true',
    cryptoKeywords: ['BTC', 'ETH', 'USDT', 'SOL', 'DOGE', 'PEPE', 'SHIB'],
    minFollowersForSignal: parseInt(process.env.TWITTER_MIN_FOLLOWERS) || 10000,
    heartbeatInterval: parseInt(process.env.TWITTER_HEARTBEAT_INTERVAL) || 30000,
    reconnectDelay: parseInt(process.env.TWITTER_RECONNECT_DELAY) || 5000
  };

  const monitor = new TwitterMonitorService(config);

  // 事件监听
  monitor.on('connected', () => {
    console.log('✅ 已连接到6551.io WebSocket');
  });

  monitor.on('disconnected', ({ code, reason }) => {
    console.log(`⚠️  断开连接: ${code} - ${reason}`);
  });

  monitor.on('error', (error) => {
    console.error('❌ 连接错误:', error.message);
  });

  monitor.on('heartbeat', () => {
    console.log('💓 心跳正常');
  });

  monitor.on('message', (message, sentimentAnalysis) => {
    console.log(`📨 收到推文: @${message.twAccount}`);
    if (sentimentAnalysis && sentimentAnalysis.isRelevant) {
      console.log(`📊 情绪分析: ${sentimentAnalysis.score.toFixed(2)} (置信度: ${sentimentAnalysis.confidence.toFixed(2)})`);
    }
  });

  monitor.on('tradingSignal', (signal) => {
    console.log('🚨 交易信号生成!');
    console.log(`📈 符号: ${signal.symbol || 'N/A'}`);
    console.log(`📊 情绪分数: ${signal.sentiment.toFixed(2)}`);
    console.log(`🎯 置信度: ${signal.confidence.toFixed(2)}`);
    console.log(`👥 影响力: ${signal.influence}`);
    console.log(`📝 内容: ${signal.metadata.content.substring(0, 100)}...`);
    console.log('─'.repeat(50));
  });

  // 优雅关闭
  process.on('SIGINT', () => {
    console.log('\n⏹️  正在停止监控服务...');
    monitor.stop();
    process.exit(0);
  });

  // 启动监控
  monitor.start();

  console.log('🎯 监控服务已启动，等待6551.io信号...');
  console.log('💡 按 Ctrl+C 停止监控');
}

// 启动服务
startTwitterMonitor().catch(error => {
  console.error('💥 启动失败:', error);
  process.exit(1);
});