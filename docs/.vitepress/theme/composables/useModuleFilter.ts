import { ref, computed, watch } from 'vue'

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
