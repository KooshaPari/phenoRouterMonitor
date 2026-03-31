import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Button } from '@/components/button'
import { AlertCircle, Check } from 'lucide-react'

export function Settings() {
  const [settings, setSettings] = useState({
    healthCheckInterval: 30,
    enableNotifications: true,
    enableDetailedLogs: false,
    maxRetries: 3,
  })
  const [saved, setSaved] = useState(false)

  const handleChange = (key: string, value: any) => {
    setSettings((prev) => ({ ...prev, [key]: value }))
    setSaved(false)
  }

  const handleSave = async () => {
    try {
      // TODO: Call API to save settings
      // await dashboardAPI.updateSettings(settings)
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch (err) {
      console.error('Failed to save settings:', err)
    }
  }

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-2xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-4xl font-bold tracking-tight">Settings</h1>
          <p className="text-muted-foreground mt-2">
            Configure dashboard behavior and health checks
          </p>
        </div>

        {/* Health Check Configuration */}
        <Card>
          <CardHeader>
            <CardTitle>Health Check Configuration</CardTitle>
            <CardDescription>
              Control how frequently the system checks service health
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <label className="block text-sm font-medium mb-2">
                Health Check Interval (seconds)
              </label>
              <input
                type="number"
                min="5"
                max="300"
                value={settings.healthCheckInterval}
                onChange={(e) =>
                  handleChange('healthCheckInterval', parseInt(e.target.value))
                }
                className="w-full px-3 py-2 border border-border rounded-md bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
              />
              <p className="text-xs text-muted-foreground mt-2">
                How often to check service health (5-300 seconds)
              </p>
            </div>

            <div className="border-t pt-6">
              <h4 className="font-semibold mb-4">Health Check Toggles</h4>
              <div className="space-y-3">
                {[
                  { key: 'enableNotifications', label: 'Enable notifications on service degradation' },
                  { key: 'enableDetailedLogs', label: 'Enable detailed health check logs' },
                ].map(({ key, label }) => (
                  <label key={key} className="flex items-center gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={settings[key as keyof typeof settings] as boolean}
                      onChange={(e) => handleChange(key, e.target.checked)}
                      className="w-4 h-4 rounded border-border"
                    />
                    <span className="text-sm">{label}</span>
                  </label>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Retry Configuration */}
        <Card>
          <CardHeader>
            <CardTitle>Retry Configuration</CardTitle>
            <CardDescription>
              Configure how the system handles transient failures
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <label className="block text-sm font-medium mb-2">
                Maximum Retry Attempts
              </label>
              <input
                type="number"
                min="1"
                max="10"
                value={settings.maxRetries}
                onChange={(e) =>
                  handleChange('maxRetries', parseInt(e.target.value))
                }
                className="w-full px-3 py-2 border border-border rounded-md bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
              />
              <p className="text-xs text-muted-foreground mt-2">
                How many times to retry failed health checks (1-10)
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Save Status */}
        <div className="flex gap-3">
          <Button onClick={handleSave} size="lg">
            Save Changes
          </Button>
          {saved && (
            <div className="flex items-center gap-2 text-green-700 dark:text-green-400">
              <Check className="h-5 w-5" />
              <span className="text-sm font-medium">Settings saved successfully</span>
            </div>
          )}
        </div>

        {/* Info Panel */}
        <Card className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
          <CardContent className="pt-6">
            <div className="flex gap-3">
              <AlertCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
              <div className="text-sm text-blue-700 dark:text-blue-300">
                <p className="font-semibold mb-1">Configuration Storage</p>
                <p>
                  Settings are stored in the backend configuration file and apply
                  to all connected dashboard instances.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
