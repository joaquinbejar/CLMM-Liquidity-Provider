import { useQuery } from '@tanstack/react-query'
import { useParams, Link } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getPool, getPoolState } from '@/lib/api'
import { formatUSD, formatPercent, shortenAddress } from '@/lib/utils'

export default function PoolDetail() {
  const { address } = useParams<{ address: string }>()

  const { data: pool, isLoading: poolLoading } = useQuery({
    queryKey: ['pool', address],
    queryFn: () => getPool(address!),
    enabled: !!address,
  })

  const { data: state } = useQuery({
    queryKey: ['pool-state', address],
    queryFn: () => getPoolState(address!),
    enabled: !!address,
    refetchInterval: 10000,
  })

  if (poolLoading) {
    return <div className="text-center py-8">Loading...</div>
  }

  if (!pool) {
    return <div className="text-center py-8">Pool not found</div>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/pools">
          <Button variant="ghost" size="icon">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <h1 className="text-3xl font-bold">
          {pool.token_a.symbol}/{pool.token_b.symbol}
        </h1>
        <span className="text-muted-foreground">
          {pool.fee_tier / 100}% fee
        </span>
      </div>

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Pool Info</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Address</span>
              <span className="font-mono text-sm">{shortenAddress(pool.address, 8)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Protocol</span>
              <span className="capitalize">{pool.protocol}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Tick Spacing</span>
              <span>{pool.tick_spacing}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Metrics</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">TVL</span>
              <span className="font-bold">{formatUSD(pool.tvl_usd)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">24h Volume</span>
              <span>{formatUSD(pool.volume_24h_usd)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Fee APY</span>
              <span className="text-green-500">{formatPercent(pool.fee_apy)}</span>
            </div>
          </CardContent>
        </Card>

        {state && (
          <Card>
            <CardHeader>
              <CardTitle>Current State</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Current Tick</span>
                <span>{state.current_tick}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Liquidity</span>
                <span className="font-mono text-sm">{state.liquidity}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token A Reserve</span>
                <span>{state.token_a_reserve}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token B Reserve</span>
                <span>{state.token_b_reserve}</span>
              </div>
            </CardContent>
          </Card>
        )}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Tokens</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="p-4 rounded-lg border">
              <div className="font-medium">{pool.token_a.symbol}</div>
              <div className="text-sm text-muted-foreground font-mono mt-1">
                {shortenAddress(pool.token_a.mint, 8)}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                {pool.token_a.decimals} decimals
              </div>
            </div>
            <div className="p-4 rounded-lg border">
              <div className="font-medium">{pool.token_b.symbol}</div>
              <div className="text-sm text-muted-foreground font-mono mt-1">
                {shortenAddress(pool.token_b.mint, 8)}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                {pool.token_b.decimals} decimals
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
