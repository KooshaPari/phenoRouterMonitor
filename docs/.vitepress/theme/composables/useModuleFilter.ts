import { ref, watch } from 'vue'

export type Module = 'all' | 'agents' | 'developers' | 'pms' | 'sdk'

export const modules: { value: Module; label: string }[] = [
  { value: 'all', label: 'All Docs' },
  { value: 'agents', label: 'For Agents' },
  { value: 'developers', label: 'For Developers' },
  { value: 'pms', label: 'For PMs' },
  { value: 'sdk', label: 'SDK / API' },
]

const STORAGE_KEY = 'agileplus-docs-module'

function loadModule(): Module {
  if (typeof localStorage === 'undefined') return 'all'
  return (localStorage.getItem(STORAGE_KEY) as Module) || 'all'
}

export const activeModule = ref<Module>(loadModule())
export const showAll = ref(false)

watch(activeModule, (v) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, v)
  }
})

export function shouldShow(audiences?: string[]): boolean {
  if (showAll.value || activeModule.value === 'all') return true
  if (!audiences || audiences.length === 0) return true
  return audiences.includes(activeModule.value)
}

/**
 * Map of page paths to their audience tags.
 * Populated at build time via transformPageData and at runtime via useData.
 */
export const pageAudiences = ref<Record<string, string[]>>({})

export function registerPageAudience(path: string, audiences: string[]) {
  pageAudiences.value[path] = audiences
}

export function shouldShowLink(link: string): boolean {
  if (showAll.value || activeModule.value === 'all') return true
  const normalized = link.replace(/^\//, '').replace(/\/$/, '')
  const audiences = pageAudiences.value[normalized]
  if (!audiences || audiences.length === 0) return true
  return audiences.includes(activeModule.value)
}
