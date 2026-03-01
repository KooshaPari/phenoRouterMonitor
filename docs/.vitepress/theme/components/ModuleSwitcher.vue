<script setup lang="ts">
import { modules, activeModule, showAll } from '../composables/useModuleFilter'
</script>

<template>
  <div class="module-switcher">
    <select v-model="activeModule" class="module-select">
      <option v-for="m in modules" :key="m.value" :value="m.value">
        {{ m.label }}
      </option>
    </select>
    <label v-if="activeModule !== 'all'" class="show-all-toggle">
      <input type="checkbox" v-model="showAll" />
      <span>Show all</span>
    </label>
  </div>
</template>

<style scoped>
.module-switcher {
  padding: 12px 16px 8px;
  border-bottom: 1px solid var(--vp-c-divider);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.module-select {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  font-size: 13px;
  font-family: var(--vp-font-family-base);
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 30px;
}

.module-select:hover {
  border-color: #7ebab5;
}

.module-select:focus {
  border-color: #7ebab5;
  box-shadow: 0 0 0 2px rgba(126, 186, 181, 0.15);
}

.module-select option {
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
}

.show-all-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--vp-c-text-3);
  cursor: pointer;
  user-select: none;
}

.show-all-toggle input {
  accent-color: #7ebab5;
  cursor: pointer;
}

.show-all-toggle:hover {
  color: var(--vp-c-text-2);
}
</style>
