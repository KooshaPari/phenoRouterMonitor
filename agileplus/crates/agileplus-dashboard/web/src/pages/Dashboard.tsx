import { useEffect, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Button } from '@/components/button'
import { EvidenceLightbox } from '@/components/EvidenceLightbox'
import { dashboardAPI, type HealthStatus, type EvidenceGalleryJson } from '@/lib/api'
import { AlertCircle, CheckCircle, Activity } from 'lucide-react'

export function Dashboard() {
  const [health, setHealth] = useState<HealthStatus | null>(null)
  const [evidence, setEvidence] = useState<EvidenceGalleryJson | null>(null)
  const [isLightboxOpen, setIsLightboxOpen] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        setLoading(true)
        const [healthRes, evidenceRes] = await Promise.all([
          dashboardAPI.getHealth(),
          dashboardAPI.getEvidenceGallery(1), // TODO: Make feature ID dynamic
        ])
        setHealth(healthRes.data)
        setEvidence(evidenceRes.data)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load dashboard')
      } finally {
        setLoading(false)
      }
    }

    fetchDashboardData()
  }, [])

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <Activity className="h-8 w-8 animate-spin mx-auto mb-2" />
          <p className="text-muted-foreground">Loading dashboard...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-7xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-4xl font-bold tracking-tight">Dashboard</h1>
          <p className="text-muted-foreground mt-2">
            Real-time feature tracking and service health monitoring
          </p>
        </div>

        {/* Error message */}
        {error && (
          <Card className="border-destructive bg-destructive/10">
            <CardContent className="pt-6">
              <div className="flex gap-3">
                <AlertCircle className="h-5 w-5 text-destructive flex-shrink-0 mt-0.5" />
                <div>
                  <p className="font-semibold text-destructive">Error</p>
                  <p className="text-sm text-destructive/80">{error}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Health status grid */}
        {health && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-2xl font-semibold">Service Health</h2>
              <div className="flex items-center gap-2">
                {health.all_healthy ? (
                  <>
                    <CheckCircle className="h-5 w-5 text-green-500" />
                    <span className="text-sm font-medium text-green-700 dark:text-green-400">
                      All systems operational
                    </span>
                  </>
                ) : (
                  <>
                    <AlertCircle className="h-5 w-5 text-yellow-500" />
                    <span className="text-sm font-medium text-yellow-700 dark:text-yellow-400">
                      Some services degraded
                    </span>
                  </>
                )}
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {health.services.map((service) => (
                <Card key={service.name}>
                  <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base">{service.name}</CardTitle>
                      {service.healthy ? (
                        <CheckCircle className="h-5 w-5 text-green-500" />
                      ) : (
                        <AlertCircle className="h-5 w-5 text-red-500" />
                      )}
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">Status</span>
                      <span className="font-medium">
                        {service.healthy ? 'Healthy' : service.degraded ? 'Degraded' : 'Unhealthy'}
                      </span>
                    </div>
                    {service.latency_ms && (
                      <div className="flex justify-between text-sm">
                        <span className="text-muted-foreground">Latency</span>
                        <span className="font-medium">{service.latency_ms}ms</span>
                      </div>
                    )}
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">Last Check</span>
                      <span className="font-medium text-xs">
                        {new Date(service.last_check).toLocaleTimeString()}
                      </span>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        )}

        {/* Evidence Gallery */}
        {evidence && evidence.artifacts.length > 0 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-2xl font-semibold">Evidence Gallery</h2>
              <Button
                onClick={() => setIsLightboxOpen(true)}
                className="gap-2"
              >
                View {evidence.artifacts.length} Artifacts
              </Button>
            </div>

            {/* Quick preview cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              {evidence.artifacts.slice(0, 4).map((artifact) => (
                <Card
                  key={artifact.id}
                  className="hover:shadow-lg transition-shadow cursor-pointer"
                  onClick={() => setIsLightboxOpen(true)}
                >
                  <CardContent className="pt-6">
                    <div className="aspect-square bg-muted rounded-md flex items-center justify-center mb-4 overflow-hidden">
                      {artifact.type_.includes('image') ? (
                        <img
                          src={artifact.url}
                          alt={artifact.title}
                          className="w-full h-full object-cover"
                        />
                      ) : (
                        <span className="text-xs text-muted-foreground">{artifact.type_}</span>
                      )}
                    </div>
                    <h4 className="font-semibold text-sm truncate">{artifact.title}</h4>
                    <p className="text-xs text-muted-foreground mt-1">
                      {new Date(artifact.created_at).toLocaleDateString()}
                    </p>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Evidence Lightbox */}
      {evidence && (
        <EvidenceLightbox
          artifacts={evidence.artifacts}
          isOpen={isLightboxOpen}
          onOpenChange={setIsLightboxOpen}
        />
      )}
    </div>
  )
}
