# CLMM LP Dashboard

A modern web dashboard for the CLMM Liquidity Provider Strategy Optimizer.

## Features

- **Dashboard**: Portfolio overview with total value, PnL, fees earned, and impermanent loss
- **Positions**: View and manage LP positions with real-time updates
- **Strategies**: Create, configure, and monitor automated LP strategies
- **Pools**: Browse available liquidity pools with metrics
- **Settings**: Configure API connection and execution settings

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **TailwindCSS** - Styling
- **React Query** - Data fetching and caching
- **React Router** - Client-side routing
- **Recharts** - Charts and visualizations
- **Lucide React** - Icons

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
# Navigate to web directory
cd web

# Install dependencies
npm install

# Start development server
npm run dev
```

The dashboard will be available at `http://localhost:3000`.

### Build for Production

```bash
npm run build
```

The built files will be in the `dist` directory.

## Configuration

The dashboard connects to the CLMM LP API server. Configure the API endpoint in the Settings page or via environment variables:

```bash
# .env.local
VITE_API_URL=http://localhost:8080
```

## Project Structure

```
web/
├── src/
│   ├── components/     # Reusable UI components
│   │   ├── ui/         # Base UI components (Button, Card, etc.)
│   │   └── Layout.tsx  # Main layout with sidebar
│   ├── hooks/          # Custom React hooks
│   ├── lib/            # Utilities and API client
│   │   ├── api.ts      # API client functions
│   │   ├── utils.ts    # Utility functions
│   │   └── websocket.ts # WebSocket client
│   ├── pages/          # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Positions.tsx
│   │   ├── PositionDetail.tsx
│   │   ├── Strategies.tsx
│   │   ├── StrategyDetail.tsx
│   │   ├── Pools.tsx
│   │   ├── PoolDetail.tsx
│   │   └── Settings.tsx
│   ├── App.tsx         # Main app component
│   ├── main.tsx        # Entry point
│   └── index.css       # Global styles
├── package.json
├── tailwind.config.js
├── tsconfig.json
└── vite.config.ts
```

## API Integration

The dashboard communicates with the CLMM LP API server via:

- **REST API**: For CRUD operations on positions, strategies, and pools
- **WebSocket**: For real-time position updates and alerts

See the API documentation at `/docs` on the running API server.

## License

MIT / Apache-2.0
