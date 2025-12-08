import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

export default function Settings() {
  const [apiKey, setApiKey] = useState('')
  const [rpcUrl, setRpcUrl] = useState('https://api.mainnet-beta.solana.com')
  const [dryRun, setDryRun] = useState(true)

  const handleSave = () => {
    // Save settings to localStorage or API
    localStorage.setItem('clmm-settings', JSON.stringify({ apiKey, rpcUrl, dryRun }))
    alert('Settings saved!')
  }

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">Settings</h1>

      <Card>
        <CardHeader>
          <CardTitle>API Configuration</CardTitle>
          <CardDescription>Configure your API connection settings</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your API key"
              className="w-full px-3 py-2 rounded-md border bg-background"
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">RPC URL</label>
            <input
              type="text"
              value={rpcUrl}
              onChange={(e) => setRpcUrl(e.target.value)}
              placeholder="https://api.mainnet-beta.solana.com"
              className="w-full px-3 py-2 rounded-md border bg-background"
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Execution Settings</CardTitle>
          <CardDescription>Configure how transactions are executed</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Dry Run Mode</div>
              <div className="text-sm text-muted-foreground">
                Simulate transactions without executing on-chain
              </div>
            </div>
            <button
              onClick={() => setDryRun(!dryRun)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                dryRun ? 'bg-primary' : 'bg-muted'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  dryRun ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Version</span>
            <span>0.1.1-alpha.2</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Build</span>
            <span>Development</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">License</span>
            <span>MIT / Apache-2.0</span>
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-end">
        <Button onClick={handleSave}>Save Settings</Button>
      </div>
    </div>
  )
}
