<script setup lang="ts">
import DefaultTheme from 'vitepress/theme'
import ModuleSwitcher from './components/ModuleSwitcher.vue'

const { Layout } = DefaultTheme
</script>

<template>
  <Layout>
    <template #sidebar-nav-before>
      <ModuleSwitcher />
    </template>
  </Layout>
</template>
