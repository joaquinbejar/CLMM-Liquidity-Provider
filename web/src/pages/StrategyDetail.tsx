import { useQuery } from '@tanstack/react-query'
import { useParams, Link } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getStrategy } from '@/lib/api'
import { formatDate } from '@/lib/utils'

export default function StrategyDetail() {
  const { id } = useParams<{ id: string }>()

  const { data: strategy, isLoading } = useQuery({
    queryKey: ['strategy', id],
    queryFn: () => getStrategy(id!),
    enabled: !!id,
  })

  if (isLoading) {
    return <div className="text-center py-8">Loading...</div>
  }

  if (!strategy) {
    return <div className="text-center py-8">Strategy not found</div>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/strategies">
          <Button variant="ghost" size="icon">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <h1 className="text-3xl font-bold">{strategy.name}</h1>
        <span className={`px-2 py-1 rounded-full text-xs font-medium ${
          strategy.running 
            ? 'bg-green-500/10 text-green-500' 
            : 'bg-muted text-muted-foreground'
        }`}>
          {strategy.running ? 'Running' : 'Stopped'}
        </span>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Configuration</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Type</span>
              <span className="capitalize">{strategy.strategy_type.replace('_', ' ')}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Pool</span>
              <span className="font-mono text-sm">{strategy.pool_address || 'Any'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Created</span>
              <span>{formatDate(strategy.created_at)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Updated</span>
              <span>{formatDate(strategy.updated_at)}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Parameters</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {strategy.parameters.rebalance_threshold_pct !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Rebalance Threshold</span>
                <span>{strategy.parameters.rebalance_threshold_pct}%</span>
              </div>
            )}
            {strategy.parameters.max_il_pct !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Max IL</span>
                <span>{strategy.parameters.max_il_pct}%</span>
              </div>
            )}
            {strategy.parameters.min_rebalance_interval_hours !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Min Rebalance Interval</span>
                <span>{strategy.parameters.min_rebalance_interval_hours}h</span>
              </div>
            )}
            {strategy.parameters.range_width_pct !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Range Width</span>
                <span>{strategy.parameters.range_width_pct}%</span>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {strategy.description && (
        <Card>
          <CardHeader>
            <CardTitle>Description</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">{strategy.description}</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
