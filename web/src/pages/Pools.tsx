import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getPools } from '@/lib/api'
import { formatUSD, formatPercent, shortenAddress } from '@/lib/utils'

export default function Pools() {
  const { data, isLoading, refetch } = useQuery({
    queryKey: ['pools'],
    queryFn: getPools,
  })

  const pools = data?.pools || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Pools</h1>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Available Pools</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading...</div>
          ) : pools.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No pools found.
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b text-left text-sm text-muted-foreground">
                    <th className="pb-3 font-medium">Pool</th>
                    <th className="pb-3 font-medium">Pair</th>
                    <th className="pb-3 font-medium">Protocol</th>
                    <th className="pb-3 font-medium text-right">TVL</th>
                    <th className="pb-3 font-medium text-right">24h Volume</th>
                    <th className="pb-3 font-medium text-right">Fee APY</th>
                  </tr>
                </thead>
                <tbody>
                  {pools.map((pool) => (
                    <tr key={pool.address} className="border-b last:border-0">
                      <td className="py-4">
                        <Link 
                          to={`/pools/${pool.address}`}
                          className="font-mono text-sm hover:text-primary"
                        >
                          {shortenAddress(pool.address)}
                        </Link>
                      </td>
                      <td className="py-4">
                        <span className="font-medium">
                          {pool.token_a.symbol}/{pool.token_b.symbol}
                        </span>
                        <span className="ml-2 text-xs text-muted-foreground">
                          {pool.fee_tier / 100}%
                        </span>
                      </td>
                      <td className="py-4 capitalize">{pool.protocol}</td>
                      <td className="py-4 text-right">{formatUSD(pool.tvl_usd)}</td>
                      <td className="py-4 text-right">{formatUSD(pool.volume_24h_usd)}</td>
                      <td className="py-4 text-right text-green-500">
                        {formatPercent(pool.fee_apy)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
