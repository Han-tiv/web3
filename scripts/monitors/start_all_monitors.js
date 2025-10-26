#!/usr/bin/env node

/**
 * 统一监控服务管理脚本
 * 管理所有社交媒体和交易监控服务
 */

require('dotenv').config();
const { spawn } = require('child_process');

class MonitorManager {
  constructor() {
    this.processes = new Map();
    this.isRunning = false;
  }

  async start() {
    console.log('🚀 启动统一监控服务管理器...');
    console.log('═'.repeat(60));

    this.isRunning = true;

    // 检查Rust交易机器人状态
    this.checkRustTradingBot();

    // 启动各种监控服务
    await this.startMonitoringServices();

    // 设置信号处理
    this.setupSignalHandlers();

    console.log('✅ 监控服务管理器已启动');
    console.log('💡 按 Ctrl+C 停止所有服务');

    // 定期状态报告
    setInterval(() => {
      this.statusReport();
    }, 300000); // 每5分钟
  }

  checkRustTradingBot() {
    console.log('🔍 检查Rust交易机器人状态...');

    try {
      const { execSync } = require('child_process');
      const output = execSync('ps aux | grep -E "(signal_trader|profit_monitor)" | grep -v grep', { encoding: 'utf8' });

      if (output.includes('signal_trader') && output.includes('profit_monitor')) {
        console.log('✅ Rust交易机器人正常运行');
        console.log('   - signal_trader: 运行中');
        console.log('   - profit_monitor: 运行中');
        console.log('   - 止损设置: -45%');
      } else {
        console.log('⚠️  Rust交易机器人未完全运行');
      }
    } catch (error) {
      console.log('❌ 无法检查Rust交易机器人状态');
    }

    console.log('');
  }

  async startMonitoringServices() {
    console.log('📡 启动监控服务...');

    // 6551.io Twitter监控
    this.startService('6551-monitor', 'node', ['start_6551_monitor.js'], {
      name: '6551.io Twitter监控',
      description: '社交媒体情绪分析',
      required_env: ['TWITTER_TOKEN']
    });

    // Telegram监控 (TypeScript 服务)
    this.startService('tg-monitor', 'node', ['start_tg_monitor.js'], {
      name: 'Telegram监控 (TypeScript)',
      description: '红包和空投机会监控',
      required_env: ['TELEGRAM_BOT_TOKEN']
    });

    // Nitter监控 (如果可用)
    if (this.checkNitterAvailable()) {
      this.startService('nitter-monitor', 'node', ['apps/social-monitor/services/nitter/dist/index.js'], {
        name: 'Nitter监控',
        description: 'Twitter RSS监控',
        required_env: []
      });
    }

    console.log('');
  }

  startService(id, command, args, config) {
    console.log(`🔄 启动 ${config.name}...`);

    // 检查必需的环境变量
    const missingEnv = config.required_env.filter(env => !process.env[env]);
    if (missingEnv.length > 0) {
      console.log(`⚠️  ${config.name} 缺少环境变量: ${missingEnv.join(', ')}`);
      console.log(`   说明: ${config.description}`);
      console.log('   状态: 配置不完整，跳过启动');
      return;
    }

    try {
      const process = spawn(command, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        cwd: '/home/hanins/code'
      });

      this.processes.set(id, {
        process,
        config,
        startTime: new Date(),
        restartCount: 0
      });

      process.stdout.on('data', (data) => {
        console.log(`[${config.name}] ${data.toString().trim()}`);
      });

      process.stderr.on('data', (data) => {
        console.error(`[${config.name} ERROR] ${data.toString().trim()}`);
      });

      process.on('exit', (code) => {
        console.log(`⚠️  ${config.name} 已退出 (代码: ${code})`);
        this.processes.delete(id);

        // 自动重启逻辑
        if (this.isRunning && code !== 0) {
          setTimeout(() => {
            console.log(`🔄 重启 ${config.name}...`);
            this.startService(id, command, args, config);
          }, 5000);
        }
      });

      console.log(`✅ ${config.name} 已启动 (PID: ${process.pid})`);

    } catch (error) {
      console.error(`❌ 启动 ${config.name} 失败:`, error.message);
    }
  }

  checkNitterAvailable() {
    try {
      const fs = require('fs');
      return fs.existsSync('/home/hanins/code/apps/social-monitor/services/nitter/dist/index.js');
    } catch {
      return false;
    }
  }

  statusReport() {
    console.log('\\n📊 服务状态报告:');
    console.log('─'.repeat(40));
    console.log(`⏰ 时间: ${new Date().toLocaleString()}`);
    console.log(`🔧 运行服务: ${this.processes.size}个`);

    for (const [id, service] of this.processes) {
      const uptime = Math.floor((Date.now() - service.startTime) / 1000);
      console.log(`   ✅ ${service.config.name}: 运行 ${uptime}s`);
    }
    console.log('');
  }

  setupSignalHandlers() {
    const shutdown = (signal) => {
      console.log(`\\n⏹️  收到 ${signal} 信号，正在关闭所有服务...`);
      this.isRunning = false;

      for (const [id, service] of this.processes) {
        console.log(`🛑 停止 ${service.config.name}...`);
        service.process.kill();
      }

      setTimeout(() => {
        console.log('✅ 所有服务已停止');
        process.exit(0);
      }, 2000);
    };

    process.on('SIGINT', () => shutdown('SIGINT'));
    process.on('SIGTERM', () => shutdown('SIGTERM'));
  }
}

// 启动管理器
const manager = new MonitorManager();
manager.start().catch(error => {
  console.error('💥 启动失败:', error);
  process.exit(1);
});
