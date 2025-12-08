import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { Plus, Play, Square, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { getStrategies, startStrategy, stopStrategy } from '@/lib/api'

export default function Strategies() {
  const queryClient = useQueryClient()
  const { data, isLoading, refetch } = useQuery({
    queryKey: ['strategies'],
    queryFn: getStrategies,
  })

  const startMutation = useMutation({
    mutationFn: startStrategy,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['strategies'] }),
  })

  const stopMutation = useMutation({
    mutationFn: stopStrategy,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['strategies'] }),
  })

  const strategies = data?.strategies || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Strategies</h1>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Create Strategy
          </Button>
        </div>
      </div>

      {isLoading ? (
        <div className="text-center py-8 text-muted-foreground">Loading...</div>
      ) : strategies.length === 0 ? (
        <Card>
          <CardContent className="py-8 text-center text-muted-foreground">
            No strategies found. Create your first strategy to automate your LP positions.
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {strategies.map((strategy) => (
            <Card key={strategy.id}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-lg">{strategy.name}</CardTitle>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                  strategy.running 
                    ? 'bg-green-500/10 text-green-500' 
                    : 'bg-muted text-muted-foreground'
                }`}>
                  {strategy.running ? 'Running' : 'Stopped'}
                </span>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="text-sm text-muted-foreground">
                  {strategy.description || 'No description'}
                </div>
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-muted-foreground">Type:</span>
                  <span className="capitalize">{strategy.strategy_type.replace('_', ' ')}</span>
                </div>
                <div className="flex gap-2">
                  <Link to={`/strategies/${strategy.id}`} className="flex-1">
                    <Button variant="outline" size="sm" className="w-full">
                      View Details
                    </Button>
                  </Link>
                  {strategy.running ? (
                    <Button 
                      variant="destructive" 
                      size="sm"
                      onClick={() => stopMutation.mutate(strategy.id)}
                      disabled={stopMutation.isPending}
                    >
                      <Square className="h-4 w-4" />
                    </Button>
                  ) : (
                    <Button 
                      variant="default" 
                      size="sm"
                      onClick={() => startMutation.mutate(strategy.id)}
                      disabled={startMutation.isPending}
                    >
                      <Play className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
