import express from 'express';
import cors from 'cors';
import path from 'path';
import { NitterService } from './index';
import { NitterUserService } from './userService';

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.static(path.join(__dirname, '..', 'public')));

// Initialize services
const nitterService = new NitterService();
const userService = new NitterUserService();

// Routes
app.get('/health', async (req, res) => {
  try {
    res.json({
      status: 'healthy',
      service: 'nitter-api',
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      version: '1.0.0'
    });
  } catch (error) {
    res.status(500).json({
      status: 'unhealthy',
      error: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

app.get('/stats', async (req, res) => {
  try {
    const stats = nitterService.getStats();
    res.json(stats);
  } catch (error) {
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Failed to get stats'
    });
  }
});

app.get('/opportunities', async (req, res) => {
  try {
    const limit = parseInt(req.query.limit as string) || 10;
    const opportunities = await nitterService.getRecentOpportunities(limit);
    res.json(opportunities);
  } catch (error) {
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Failed to get opportunities'
    });
  }
});

app.get('/filters', async (req, res) => {
  try {
    // This would come from the TwitterFilters instance
    const filterStats = {
      totalRules: 6,
      accountTiers: [
        { tier: 'premium', accountCount: 14, priorityMultiplier: 2.0 },
        { tier: 'verified', accountCount: 10, priorityMultiplier: 1.5 },
        { tier: 'standard', accountCount: 7, priorityMultiplier: 1.2 }
      ],
      suspiciousPatternsCount: 9,
      codePatternCount: 5
    };
    res.json(filterStats);
  } catch (error) {
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Failed to get filter config'
    });
  }
});

app.get('/accounts', async (req, res) => {
  try {
    const accounts = nitterService.getMonitoredAccounts();
    res.json({ accounts, count: accounts.length });
  } catch (error) {
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Failed to get accounts'
    });
  }
});

app.get('/keywords', async (req, res) => {
  try {
    const keywords = nitterService.getKeywords();
    res.json({ keywords, count: keywords.length });
  } catch (error) {
    res.status(500).json({
      error: error instanceof Error ? error.message : 'Failed to get keywords'
    });
  }
});

// User-related endpoints
app.get('/user/:username/following', async (req, res) => {
  try {
    const { username } = req.params;
    const limit = parseInt(req.query.limit as string) || 50;

    console.log(`Getting following list for @${username}`);
    const result = await userService.getUserFollowing(username, limit);

    res.json({
      success: true,
      data: result,
      message: `Successfully retrieved ${result.following.length} following accounts for @${username}`
    });
  } catch (error) {
    console.error(`Error getting following for @${req.params.username}:`, error);
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to get user following',
      data: null
    });
  }
});

app.get('/user/:username/profile', async (req, res) => {
  try {
    const { username } = req.params;

    console.log(`Getting profile for @${username}`);
    const profile = await userService.getUserProfile(username);

    if (!profile) {
      return res.status(404).json({
        success: false,
        error: 'User profile not found',
        data: null
      });
    }

    res.json({
      success: true,
      data: profile,
      message: `Successfully retrieved profile for @${username}`
    });
    return;
  } catch (error) {
    console.error(`Error getting profile for @${req.params.username}:`, error);
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to get user profile',
      data: null
    });
    return;
  }
});

app.get('/user/:username/tweets', async (req, res) => {
  try {
    const { username } = req.params;
    const limit = parseInt(req.query.limit as string) || 20;

    console.log(`Getting tweets for @${username}`);
    const tweets = await userService.getUserTweets(username, limit);

    res.json({
      success: true,
      data: {
        username,
        tweets,
        count: tweets.length
      },
      message: `Successfully retrieved ${tweets.length} tweets for @${username}`
    });
  } catch (error) {
    console.error(`Error getting tweets for @${req.params.username}:`, error);
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to get user tweets',
      data: null
    });
  }
});

// Dashboard route
app.get('/dashboard', (req, res) => {
  res.sendFile(path.join(__dirname, '..', 'public', 'dashboard.html'));
});

// Diagnostic route
app.get('/diagnostic', (req, res) => {
  res.sendFile(path.join(__dirname, '..', 'public', 'diagnostic.html'));
});

// Connection test route
app.get('/test', (req, res) => {
  res.sendFile(path.join(__dirname, '..', 'public', 'test.html'));
});

// Mobile optimized route
app.get('/mobile', (req, res) => {
  res.sendFile(path.join(__dirname, '..', 'public', 'mobile.html'));
});

app.get('/', (req, res) => {
  res.redirect('/dashboard');
});

// Error handling
app.use((err: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
  console.error('API Error:', err);
  res.status(500).json({
    error: 'Internal server error',
    message: err.message
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: 'Not found',
    message: `Route ${req.method} ${req.url} not found`
  });
});

// Start server
async function startAPIServer() {
  try {
    // Start the Nitter monitoring service
    await nitterService.start();

    // Start the HTTP API server
    app.listen(PORT, () => {
      console.log(`🚀 Nitter API Server running on http://localhost:${PORT}`);
      console.log(`📊 Dashboard available at http://localhost:${PORT}/dashboard`);
      console.log(`🔗 Health check at http://localhost:${PORT}/health`);
    });
  } catch (error) {
    console.error('Failed to start API server:', error);
    process.exit(1);
  }
}

export { app, startAPIServer };

// Start if called directly
if (require.main === module) {
  startAPIServer();
}