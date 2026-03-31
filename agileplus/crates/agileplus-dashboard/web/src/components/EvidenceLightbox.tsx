import { useState } from 'react'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from './dialog'
import { Button } from './button'
import { Card } from './card'
import { Image as ImageIcon, Download, Maximize2 } from 'lucide-react'
import type { EvidenceArtifactJson } from '@/lib/api'

interface EvidenceLightboxProps {
  artifacts: EvidenceArtifactJson[]
  isOpen: boolean
  onOpenChange: (open: boolean) => void
}

export function EvidenceLightbox({ artifacts, isOpen, onOpenChange }: EvidenceLightboxProps) {
  const [selectedArtifact, setSelectedArtifact] = useState<EvidenceArtifactJson | null>(
    artifacts.length > 0 ? artifacts[0] : null
  )

  if (!selectedArtifact && artifacts.length > 0) {
    setSelectedArtifact(artifacts[0])
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Evidence Gallery</DialogTitle>
          <DialogDescription>
            View and download test evidence, artifacts, and reports
          </DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          {/* Preview pane */}
          <div className="lg:col-span-2">
            {selectedArtifact && (
              <Card className="bg-muted">
                <div className="p-6 flex flex-col gap-4">
                  <div className="aspect-video bg-black rounded-md flex items-center justify-center overflow-hidden">
                    {selectedArtifact.type_.includes('image') ? (
                      <img
                        src={selectedArtifact.url}
                        alt={selectedArtifact.title}
                        className="max-w-full max-h-full object-contain"
                      />
                    ) : (
                      <div className="flex flex-col items-center gap-2 text-muted-foreground">
                        <ImageIcon className="h-12 w-12" />
                        <span className="text-sm">{selectedArtifact.type_}</span>
                      </div>
                    )}
                  </div>
                  <div>
                    <h3 className="font-semibold text-lg mb-2">{selectedArtifact.title}</h3>
                    <p className="text-sm text-muted-foreground mb-4">{selectedArtifact.path}</p>
                    <div className="flex gap-2">
                      <Button
                        variant="default"
                        size="sm"
                        className="gap-2"
                        asChild
                      >
                        <a href={selectedArtifact.url} download>
                          <Download className="h-4 w-4" />
                          Download
                        </a>
                      </Button>
                      <Button variant="outline" size="sm" className="gap-2">
                        <Maximize2 className="h-4 w-4" />
                        Full View
                      </Button>
                    </div>
                  </div>
                </div>
              </Card>
            )}
          </div>

          {/* Thumbnail gallery */}
          <div className="lg:col-span-1">
            <div className="space-y-2">
              <h4 className="font-semibold text-sm mb-3">Artifacts ({artifacts.length})</h4>
              <div className="grid grid-cols-2 lg:grid-cols-1 gap-2 max-h-[400px] overflow-y-auto">
                {artifacts.map((artifact) => (
                  <button
                    key={artifact.id}
                    onClick={() => setSelectedArtifact(artifact)}
                    className={`p-3 rounded-md border-2 transition-all hover:bg-accent hover:border-accent ${
                      selectedArtifact?.id === artifact.id
                        ? 'border-primary bg-primary/10'
                        : 'border-border'
                    }`}
                  >
                    <div className="text-left">
                      <div className="text-xs font-semibold truncate">{artifact.title}</div>
                      <div className="text-xs text-muted-foreground">{artifact.type_}</div>
                      <div className="text-xs text-muted-foreground mt-1">
                        {new Date(artifact.created_at).toLocaleDateString()}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
