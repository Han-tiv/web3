const express = require('express');
const HealthChecker = require('../lib/health-checker');

const app = express();
const port = process.env.HEALTH_PORT || 3001;

const healthChecker = new HealthChecker();

// Health check routes
app.get('/health', healthChecker.health.bind(healthChecker));
app.get('/health/detailed', healthChecker.detailedHealth.bind(healthChecker));
app.get('/ready', healthChecker.ready.bind(healthChecker));
app.get('/live', healthChecker.live.bind(healthChecker));

// Simple status endpoint
app.get('/status', (req, res) => {
  res.json({
    service: 'social-monitor',
    status: 'running',
    timestamp: new Date().toISOString(),
    monitoring: {
      twitter: process.env.TWITTER_ENABLED === 'true',
      telegram: process.env.TELEGRAM_ENABLED === 'true',
      discord: process.env.DISCORD_ENABLED === 'true'
    }
  });
});

app.listen(port, () => {
  console.log(`Social Monitor health check server running on port ${port}`);
});

module.exports = app;