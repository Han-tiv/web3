import { TwitterFilters } from '../src/filters';
import { Tweet } from '../src/index';

// Test data
const testTweets: Tweet[] = [
  {
    id: '1234567890',
    content: '🧧 Binance红包来了！BP123456789，先到先得！关注+转发+点赞即可获得10USDT红包 #Binance #RedPacket',
    author: 'binance',
    timestamp: new Date(),
    url: 'https://twitter.com/binance/status/1234567890',
    retweetsCount: 500,
    likesCount: 1200
  },
  {
    id: '1234567891',
    content: 'New airdrop opportunity! Follow @uniswap and complete the quiz to earn 50 UNI tokens. Limited time offer!',
    author: 'uniswap',
    timestamp: new Date(),
    url: 'https://twitter.com/uniswap/status/1234567891',
    retweetsCount: 200,
    likesCount: 800
  },
  {
    id: '1234567892',
    content: 'Check out this amazing opportunity to double your crypto! Send 1 BTC get 2 BTC back guaranteed!',
    author: 'fake_account',
    timestamp: new Date(),
    url: 'https://twitter.com/fake_account/status/1234567892',
    retweetsCount: 10,
    likesCount: 5
  },
  {
    id: '1234567893',
    content: 'Learn and earn with Coinbase! Complete our cryptocurrency course and earn $3 in Bitcoin. No purchase necessary.',
    author: 'coinbase',
    timestamp: new Date(Date.now() - 60000), // 1 minute ago
    url: 'https://twitter.com/coinbase/status/1234567893',
    retweetsCount: 300,
    likesCount: 600
  },
  {
    id: '1234567894',
    content: 'Old red packet from yesterday: BP987654321',
    author: 'binance',
    timestamp: new Date(Date.now() - 25 * 60 * 60 * 1000), // 25 hours ago
    url: 'https://twitter.com/binance/status/1234567894',
    retweetsCount: 100,
    likesCount: 200
  }
];

async function testTwitterFilters() {
  console.log('🔍 Testing Twitter Filters System...\n');

  const filters = new TwitterFilters();

  // Test filtering stats
  console.log('📊 Filter Statistics:');
  console.log(JSON.stringify(filters.getFilteringStats(), null, 2));
  console.log('\n');

  // Test each tweet
  for (let i = 0; i < testTweets.length; i++) {
    const tweet = testTweets[i];
    console.log(`🐦 Testing Tweet ${i + 1}:`);
    console.log(`Content: ${tweet.content.substring(0, 80)}...`);
    console.log(`Author: @${tweet.author}`);
    console.log(`Age: ${Math.round((Date.now() - tweet.timestamp.getTime()) / 1000 / 60)} minutes ago`);

    const opportunity = filters.analyzeOpportunity(tweet);

    if (opportunity) {
      console.log('✅ Detected Opportunity:');
      console.log(`   Type: ${opportunity.type}`);
      console.log(`   Priority: ${opportunity.priority}/10`);
      console.log(`   Title: ${opportunity.title}`);
      console.log(`   Requirements: ${opportunity.requirements.join(', ')}`);
      if (opportunity.reward) console.log(`   Reward: ${opportunity.reward}`);
      if (opportunity.metadata.codes?.length) {
        console.log(`   Codes: ${opportunity.metadata.codes.join(', ')}`);
      }
    } else {
      console.log('❌ No opportunity detected (filtered out)');
    }
    console.log('---\n');
  }

  // Test edge cases
  console.log('🧪 Testing Edge Cases:\n');

  // Test suspicious content
  const suspiciousTweet: Tweet = {
    id: 'suspicious',
    content: 'Send me 0.1 BTC and get 1 BTC back guaranteed! Risk free investment!',
    author: 'scammer',
    timestamp: new Date(),
    url: 'https://twitter.com/scammer/status/suspicious'
  };

  console.log('Testing suspicious content detection...');
  const suspiciousResult = filters.analyzeOpportunity(suspiciousTweet);
  console.log(suspiciousResult ? '❌ FAILED - Scam not detected!' : '✅ PASSED - Scam filtered out');

  console.log('\n🎉 Twitter Filters Testing Complete!');
}

// Run tests
testTwitterFilters().catch(console.error);