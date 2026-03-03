<script setup lang="ts">
import { watch, onMounted, nextTick } from 'vue'
import { useRouter } from 'vitepress'
import { activeModule, showAll } from '../composables/useModuleFilter'

// Static audience map: path → audiences
// Built from frontmatter at page level, but for sidebar filtering we need
// all pages' audiences available client-side. We hardcode the map here.
const audienceMap: Record<string, string[]> = {
  // Introduction
  'guide/getting-started': ['developers', 'agents', 'pms'],
  'guide/quick-start': ['developers', 'agents'],
  // Concepts
  'concepts/spec-driven-dev': ['developers', 'agents', 'pms'],
  'concepts/governance': ['developers', 'agents', 'pms'],
  'concepts/agent-dispatch': ['agents', 'developers'],
  'concepts/feature-lifecycle': ['developers', 'pms', 'agents'],
  // Agents
  'agents/prompt-format': ['agents'],
  'agents/subcommand-reference': ['agents'],
  'agents/governance-constraints': ['agents'],
  'agents/harness-integration': ['agents', 'developers'],
  // Developers
  'developers/contributing': ['developers'],
  'developers/extending': ['developers'],
  'developers/testing': ['developers'],
  // SDK
  'sdk/grpc-api': ['sdk', 'developers'],
  'sdk/mcp-tools': ['sdk', 'agents'],
  'sdk/storage-port': ['sdk', 'developers'],
  'sdk/vcs-port': ['sdk', 'developers'],
  // Guide
  'guide/init': ['developers'],
  'guide/workflow': ['developers', 'agents', 'pms'],
  'guide/triage': ['developers', 'pms'],
  'guide/configuration': ['developers'],
  'guide/sync': ['developers', 'pms'],
  // Architecture
  'architecture/overview': ['developers', 'sdk'],
  'architecture/domain-model': ['developers', 'sdk'],
  'architecture/ports': ['developers', 'sdk'],
  // Workflow Phases
  'workflow/specify': ['developers', 'agents', 'pms'],
  'workflow/clarify': ['developers', 'agents', 'pms'],
  'workflow/research': ['developers', 'agents'],
  'workflow/plan': ['developers', 'agents', 'pms'],
  'workflow/tasks': ['developers', 'agents', 'pms'],
  'workflow/implement': ['developers', 'agents'],
  'workflow/review': ['developers', 'agents'],
  'workflow/accept': ['developers', 'pms'],
  'workflow/merge': ['developers'],
  // Process
  'process/constitution': ['developers', 'pms'],
  'process/checklists': ['developers', 'pms', 'agents'],
  'process/analyze': ['developers', 'pms'],
  'process/retrospective': ['developers', 'pms'],
  'process/status-dashboard': ['developers', 'pms', 'agents'],
  // Roadmap
  'roadmap/': ['pms', 'developers'],
  'roadmap/release-notes': ['pms', 'developers'],
  // Reference
  'reference/cli': ['developers', 'agents'],
  'reference/crates': ['developers', 'sdk'],
  'reference/subcommands': ['agents', 'developers'],
  'reference/env-vars': ['developers', 'agents'],
  // Examples
  'examples/full-pipeline': ['developers', 'agents', 'pms'],
  'examples/triage-workflow': ['developers', 'pms'],
  'examples/agent-integration': ['agents', 'developers'],
  // Doc System
  'doc-system/layers': ['developers', 'pms'],
  'doc-system/frontmatter': ['developers', 'agents'],
  'doc-system/federation': ['developers'],
}

function filterSidebar() {
  if (typeof document === 'undefined') return
  const mod = activeModule.value
  const all = showAll.value || mod === 'all'

  // Find all sidebar link items
  const sidebarItems = document.querySelectorAll('.VPSidebarItem.level-1')
  sidebarItems.forEach((item) => {
    const link = item.querySelector('a')
    if (!link) return
    const href = link.getAttribute('href') || ''
    // Normalize: remove base path, leading/trailing slashes
    const path = href.replace(/^\/AgilePlus\//, '/').replace(/^\//, '').replace(/\.html$/, '').replace(/\/$/, '')

    if (all) {
      ;(item as HTMLElement).style.display = ''
      return
    }

    const audiences = audienceMap[path]
    if (!audiences || audiences.length === 0) {
      // No audience restriction — always show
      ;(item as HTMLElement).style.display = ''
      return
    }

    ;(item as HTMLElement).style.display = audiences.includes(mod) ? '' : 'none'
  })

  // Hide empty groups (groups where all children are hidden)
  const groups = document.querySelectorAll('.VPSidebarItem.level-0')
  groups.forEach((group) => {
    const children = group.querySelectorAll('.VPSidebarItem.level-1')
    if (children.length === 0) return
    const visibleCount = Array.from(children).filter(
      (c) => (c as HTMLElement).style.display !== 'none'
    ).length
    ;(group as HTMLElement).style.display = visibleCount === 0 ? 'none' : ''
  })
}

const router = useRouter()

onMounted(() => {
  nextTick(filterSidebar)
  // Re-filter on route change (sidebar may re-render)
  router.onAfterRouteChanged = () => {
    nextTick(filterSidebar)
  }
})

watch([activeModule, showAll], () => {
  nextTick(filterSidebar)
})
</script>

<template>
  <span style="display: none" />
</template>
