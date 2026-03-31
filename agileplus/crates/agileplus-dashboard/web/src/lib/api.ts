import axios from 'axios'

const API_BASE_URL = '/api'

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// API Types (from Rust backend routes)
export interface ServiceHealthJson {
  name: string
  healthy: boolean
  degraded: boolean
  latency_ms?: number
  last_check: string
}

export interface HealthStatus {
  services: ServiceHealthJson[]
  timestamp: string
  all_healthy: boolean
}

export interface EvidenceArtifactJson {
  id: string
  type_: string
  title: string
  path: string
  url: string
  created_at: string
}

export interface EvidenceGalleryJson {
  feature_id: string
  artifacts: EvidenceArtifactJson[]
  generated_at?: string
}

export interface AgentInfo {
  name: string
  status: string
  current_task: string
  pid?: number
  started_at?: string
  worktree: string
  uptime: string
}

// API Endpoints
export const dashboardAPI = {
  // Health check
  getHealth: () => api.get<HealthStatus>('/dashboard/health'),

  // Features
  getFeatures: (projectId?: number) => {
    const params = projectId ? `?project_id=${projectId}` : ''
    return api.get(`/dashboard/features${params}`)
  },

  // Feature details
  getFeatureDetail: (featureId: number) =>
    api.get(`/dashboard/features/${featureId}`),

  // Evidence gallery
  getEvidenceGallery: (featureId: number) =>
    api.get<EvidenceGalleryJson>(`/dashboard/features/${featureId}/evidence`),

  // Agent info (Phase 3 feature, stub for now)
  getAgents: () => api.get<AgentInfo[]>('/dashboard/agents'),

  // Settings/config
  getSettings: () => api.get('/dashboard/settings'),
  updateSettings: (settings: Record<string, unknown>) =>
    api.post('/dashboard/settings', settings),
}

export default api
