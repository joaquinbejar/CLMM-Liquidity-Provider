import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useParams, Link } from 'react-router-dom'
import { ArrowLeft, RefreshCw, X, DollarSign } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getPosition, closePosition, collectFees } from '@/lib/api'
import { formatUSD, formatPercent, shortenAddress, formatDate } from '@/lib/utils'

export default function PositionDetail() {
  const { address } = useParams<{ address: string }>()
  const queryClient = useQueryClient()

  const { data: position, isLoading } = useQuery({
    queryKey: ['position', address],
    queryFn: () => getPosition(address!),
    enabled: !!address,
  })

  const closeMutation = useMutation({
    mutationFn: () => closePosition(address!),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['positions'] }),
  })

  const collectMutation = useMutation({
    mutationFn: () => collectFees(address!),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['position', address] }),
  })

  if (isLoading) {
    return <div className="text-center py-8">Loading...</div>
  }

  if (!position) {
    return <div className="text-center py-8">Position not found</div>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/positions">
          <Button variant="ghost" size="icon">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <h1 className="text-3xl font-bold">Position Details</h1>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Position Info</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Address</span>
              <span className="font-mono">{shortenAddress(position.address, 8)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Pool</span>
              <span className="font-mono">{shortenAddress(position.pool_address, 8)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Tick Range</span>
              <span>{position.tick_lower} → {position.tick_upper}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Liquidity</span>
              <span>{position.liquidity}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">In Range</span>
              <span className={position.in_range ? 'text-green-500' : 'text-yellow-500'}>
                {position.in_range ? 'Yes' : 'No'}
              </span>
            </div>
            {position.created_at && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span>{formatDate(position.created_at)}</span>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Performance</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Value</span>
              <span className="font-bold">{formatUSD(position.value_usd)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Net PnL</span>
              <span className={parseFloat(position.pnl.net_pnl_pct) >= 0 ? 'text-green-500' : 'text-red-500'}>
                {formatUSD(position.pnl.net_pnl_usd)} ({formatPercent(position.pnl.net_pnl_pct)})
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Fees Earned</span>
              <span className="text-green-500">{formatUSD(position.pnl.fees_earned_usd)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Impermanent Loss</span>
              <span className="text-yellow-500">{formatPercent(position.pnl.il_pct)}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Actions</CardTitle>
        </CardHeader>
        <CardContent className="flex gap-4">
          <Button onClick={() => collectMutation.mutate()} disabled={collectMutation.isPending}>
            <DollarSign className="h-4 w-4 mr-2" />
            {collectMutation.isPending ? 'Collecting...' : 'Collect Fees'}
          </Button>
          <Button variant="outline">
            <RefreshCw className="h-4 w-4 mr-2" />
            Rebalance
          </Button>
          <Button 
            variant="destructive" 
            onClick={() => closeMutation.mutate()}
            disabled={closeMutation.isPending}
          >
            <X className="h-4 w-4 mr-2" />
            {closeMutation.isPending ? 'Closing...' : 'Close Position'}
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
