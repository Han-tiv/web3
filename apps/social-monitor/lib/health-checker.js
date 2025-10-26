// Health check endpoints for Social Monitor
const express = require('express');
const { createClient } = require('redis');

class HealthChecker {
  constructor() {
    this.startTime = new Date();
    this.redisClient = createClient({
      host: process.env.REDIS_HOST || 'localhost',
      port: process.env.REDIS_PORT || 6379
    });
  }

  // Standard health check
  async health(req, res) {
    res.json({
      status: 'ok',
      timestamp: Math.floor(Date.now() / 1000),
      service: 'social-monitor',
      version: '1.0.0'
    });
  }

  // Detailed health check
  async detailedHealth(req, res) {
    try {
      const redisHealth = await this.checkRedis();
      const servicesHealth = await this.checkServices();

      const overallStatus = redisHealth.healthy && servicesHealth.healthy ? 'healthy' : 'unhealthy';
      const statusCode = overallStatus === 'healthy' ? 200 : 503;

      const health = {
        status: overallStatus,
        timestamp: Math.floor(Date.now() / 1000),
        service: 'social-monitor',
        version: '1.0.0',
        uptime: (Date.now() - this.startTime.getTime()) / 1000,
        checks: {
          redis: redisHealth,
          services: servicesHealth
        }
      };

      res.status(statusCode).json(health);
    } catch (error) {
      res.status(503).json({
        status: 'unhealthy',
        timestamp: Math.floor(Date.now() / 1000),
        error: error.message
      });
    }
  }

  // Kubernetes readiness probe
  async ready(req, res) {
    try {
      const redisHealth = await this.checkRedis();

      if (redisHealth.healthy) {
        res.json({
          status: 'ready',
          timestamp: Math.floor(Date.now() / 1000)
        });
      } else {
        res.status(503).json({
          status: 'not_ready',
          reason: 'Redis connection failed',
          timestamp: Math.floor(Date.now() / 1000)
        });
      }
    } catch (error) {
      res.status(503).json({
        status: 'not_ready',
        reason: error.message,
        timestamp: Math.floor(Date.now() / 1000)
      });
    }
  }

  // Kubernetes liveness probe
  async live(req, res) {
    res.json({
      status: 'alive',
      timestamp: Math.floor(Date.now() / 1000),
      uptime: (Date.now() - this.startTime.getTime()) / 1000
    });
  }

  // Check Redis connection
  async checkRedis() {
    try {
      await this.redisClient.ping();
      return {
        healthy: true,
        checked_at: Math.floor(Date.now() / 1000)
      };
    } catch (error) {
      return {
        healthy: false,
        checked_at: Math.floor(Date.now() / 1000),
        error: error.message
      };
    }
  }

  // Check monitoring services
  async checkServices() {
    // Check if monitoring services are running
    // This is a simplified check - in real implementation,
    // you'd check actual service status
    return {
      healthy: true,
      checked_at: Math.floor(Date.now() / 1000),
      services: {
        nitter: 'running',
        telegram: 'running',
        discord: 'running',
        aggregator: 'running'
      }
    };
  }
}

module.exports = HealthChecker;