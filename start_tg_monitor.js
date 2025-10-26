#!/usr/bin/env node

/**
 * Telegram监控服务启动脚本 (TypeScript 版本)
 */

require('dotenv').config();
const { spawn } = require('child_process');
const path = require('path');

async function startTelegramMonitor() {
  console.log('🤖 启动Telegram监控服务...');

  // 检查必需的环境变量
  if (!process.env.TELEGRAM_BOT_TOKEN) {
    console.error('❌ TELEGRAM_BOT_TOKEN未设置，请在.env文件中配置');
    console.log('💡 需要从 @BotFather 获取Bot Token');
    console.log('📝 步骤：');
    console.log('   1. 私信 @BotFather');
    console.log('   2. 发送 /newbot');
    console.log('   3. 按提示创建Bot');
    console.log('   4. 复制Token到.env文件');
    process.exit(1);
  }

  if (!process.env.TELEGRAM_MONITOR_ENABLED || process.env.TELEGRAM_MONITOR_ENABLED !== 'true') {
    console.log('⏸️ Telegram监控已禁用');
    process.exit(0);
  }

  console.log('🚀 启动Telegram监控 (TypeScript 服务)...');

  try {
    const serviceCwd = path.join(__dirname, 'apps/social-monitor/services/telegram');
    const monitorProcess = spawn('pnpm', ['dev'], {
      stdio: 'inherit',
      cwd: serviceCwd
    });

    monitorProcess.on('error', (error) => {
      console.error('💥 启动Telegram监控服务失败:', error.message);
      process.exit(1);
    });

    monitorProcess.on('exit', (code) => {
      console.log(`⚠️ Telegram监控服务退出，代码: ${code}`);
      process.exit(code);
    });

    // 优雅关闭
    process.on('SIGINT', () => {
      console.log('\n⏹️ 正在停止Telegram监控...');
      monitorProcess.kill('SIGINT');
    });

    process.on('SIGTERM', () => {
      console.log('\n⏹️ 正在停止Telegram监控...');
      monitorProcess.kill('SIGTERM');
    });

  } catch (error) {
    console.error('💥 启动失败:', error.message);
    process.exit(1);
  }
}

// 启动服务
startTelegramMonitor().catch(error => {
  console.error('💥 启动失败:', error);
  process.exit(1);
});
