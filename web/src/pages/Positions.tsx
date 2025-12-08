import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { Plus, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getPositions } from '@/lib/api'
import { formatUSD, formatPercent, shortenAddress } from '@/lib/utils'

export default function Positions() {
  const { data, isLoading, refetch } = useQuery({
    queryKey: ['positions'],
    queryFn: getPositions,
  })

  const positions = data?.positions || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Positions</h1>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Open Position
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>All Positions</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading...</div>
          ) : positions.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No positions found. Open your first position to get started.
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b text-left text-sm text-muted-foreground">
                    <th className="pb-3 font-medium">Position</th>
                    <th className="pb-3 font-medium">Pool</th>
                    <th className="pb-3 font-medium">Range</th>
                    <th className="pb-3 font-medium text-right">Value</th>
                    <th className="pb-3 font-medium text-right">PnL</th>
                    <th className="pb-3 font-medium text-right">Fees</th>
                    <th className="pb-3 font-medium text-center">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {positions.map((position) => (
                    <tr key={position.address} className="border-b last:border-0">
                      <td className="py-4">
                        <Link 
                          to={`/positions/${position.address}`}
                          className="font-medium hover:text-primary"
                        >
                          {shortenAddress(position.address)}
                        </Link>
                      </td>
                      <td className="py-4 text-muted-foreground">
                        {shortenAddress(position.pool_address)}
                      </td>
                      <td className="py-4">
                        <span className="text-sm">
                          {position.tick_lower} → {position.tick_upper}
                        </span>
                      </td>
                      <td className="py-4 text-right font-medium">
                        {formatUSD(position.value_usd)}
                      </td>
                      <td className={`py-4 text-right ${
                        parseFloat(position.pnl.net_pnl_pct) >= 0 ? 'text-green-500' : 'text-red-500'
                      }`}>
                        {formatPercent(position.pnl.net_pnl_pct)}
                      </td>
                      <td className="py-4 text-right text-green-500">
                        {formatUSD(position.pnl.fees_earned_usd)}
                      </td>
                      <td className="py-4 text-center">
                        <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium ${
                          position.status === 'active' 
                            ? 'bg-green-500/10 text-green-500' 
                            : position.status === 'pending'
                            ? 'bg-yellow-500/10 text-yellow-500'
                            : 'bg-muted text-muted-foreground'
                        }`}>
                          <span className={`h-1.5 w-1.5 rounded-full ${
                            position.status === 'active' 
                              ? 'bg-green-500' 
                              : position.status === 'pending'
                              ? 'bg-yellow-500'
                              : 'bg-muted-foreground'
                          }`} />
                          {position.status}
                        </span>
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
