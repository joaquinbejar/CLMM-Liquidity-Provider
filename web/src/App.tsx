import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Toaster } from '@/components/ui/toaster'
import Layout from '@/components/Layout'
import Dashboard from '@/pages/Dashboard'
import Positions from '@/pages/Positions'
import PositionDetail from '@/pages/PositionDetail'
import Strategies from '@/pages/Strategies'
import StrategyDetail from '@/pages/StrategyDetail'
import Pools from '@/pages/Pools'
import PoolDetail from '@/pages/PoolDetail'
import Settings from '@/pages/Settings'

function App() {
  return (
    <BrowserRouter>
      <div className="dark min-h-screen bg-background">
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="positions" element={<Positions />} />
            <Route path="positions/:address" element={<PositionDetail />} />
            <Route path="strategies" element={<Strategies />} />
            <Route path="strategies/:id" element={<StrategyDetail />} />
            <Route path="pools" element={<Pools />} />
            <Route path="pools/:address" element={<PoolDetail />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
        <Toaster />
      </div>
    </BrowserRouter>
  )
}

export default App
