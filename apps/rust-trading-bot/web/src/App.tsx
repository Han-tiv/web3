import clsx from 'clsx';
import { BrowserRouter, NavLink, Route, Routes } from 'react-router-dom';
import { SWRConfig } from 'swr';
import { Bot, Github } from 'lucide-react';

import { AIAnalysisPanel } from './components/AIAnalysisPanel';
import { BackendStatus } from './components/BackendStatus';
import { EquityChart } from './components/EquityChart';
import { PositionsList } from './components/PositionsList';
import { TelegramSignals } from './components/TelegramSignals';
import { TradesHistory } from './components/TradesHistory';

type RouteDefinition = {
  path: string;
  label: string;
  icon: string;
  element: JSX.Element;
};

function DashboardPage() {
  return (
    <>
      <section className="container mx-auto px-6 py-6">
        <BackendStatus />
      </section>

      <main className="container mx-auto px-6 py-8">
        <div className="space-y-8">
          <EquityChart />
          <PositionsList />
          <TradesHistory />
        </div>
      </main>
    </>
  );
}

function AIAnalysisPage() {
  return (
    <main className="container mx-auto px-6 py-8">
      <AIAnalysisPanel />
    </main>
  );
}

function TelegramSignalsPage() {
  return (
    <main className="container mx-auto px-6 py-8">
      <TelegramSignals />
    </main>
  );
}

const ROUTES: RouteDefinition[] = [
  {
    path: '/',
    label: '仪表盘',
    icon: '📊',
    element: <DashboardPage />,
  },
  {
    path: '/ai-analysis',
    label: 'AI分析',
    icon: '🤖',
    element: <AIAnalysisPage />,
  },
  {
    path: '/telegram-signals',
    label: 'Telegram信号',
    icon: '📡',
    element: <TelegramSignalsPage />,
  },
];

function App() {
  return (
    <BrowserRouter>
      <SWRConfig
        value={{
          revalidateOnFocus: false,
          dedupingInterval: 10000,
        }}
      >
        <div className="min-h-screen binance-bg flex flex-col">
          {/* 顶部导航 */}
          <header className="binance-card border-b border-gray-800">
            <div className="container mx-auto px-6 py-4">
              <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                <div className="flex items-center gap-3">
                  <Bot className="w-8 h-8 binance-gold" />
                  <div>
                    <h1 className="text-xl font-bold binance-text-primary">
                      AI交易机器人监控
                    </h1>
                    <p className="text-sm binance-text-secondary">
                      实时监控交易状态和盈亏数据
                    </p>
                  </div>
                </div>

                <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:gap-6">
                  <nav className="flex flex-wrap items-center gap-2">
                    {ROUTES.map((route) => (
                      <NavLink
                        key={route.path}
                        to={route.path}
                        end={route.path === '/'}
                        className={({ isActive }) =>
                          clsx(
                            'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors border',
                            isActive
                              ? 'bg-gray-800/80 binance-text-primary border-gray-700 shadow-inner'
                              : 'binance-text-secondary border-transparent hover:bg-gray-800/50 hover:text-white',
                          )
                        }
                      >
                        <span aria-hidden="true" className="text-base">
                          {route.icon}
                        </span>
                        <span>{route.label}</span>
                      </NavLink>
                    ))}
                  </nav>

                  <a
                    href="https://github.com/yourusername/rust-trading-bot"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 px-4 py-2 rounded hover:bg-gray-800 transition-colors binance-text-secondary"
                  >
                    <Github className="w-5 h-5" />
                    <span className="hidden md:inline">GitHub</span>
                  </a>
                </div>
              </div>
            </div>
          </header>

          <div className="flex-1">
            <Routes>
              {ROUTES.map(({ path, element }) => (
                <Route key={path} path={path} element={element} />
              ))}
              <Route path="*" element={<DashboardPage />} />
            </Routes>
          </div>

          {/* 页脚 */}
          <footer className="mt-16 py-8 border-t border-gray-800">
            <div className="container mx-auto px-6 text-center binance-text-secondary text-sm">
              <p>© 2024 AI Trading Bot. 仅供学习研究使用,交易有风险,投资需谨慎。</p>
            </div>
          </footer>
        </div>
      </SWRConfig>
    </BrowserRouter>
  );
}

export default App;
