<script setup lang="ts">
import { useData, withBase } from 'vitepress'
import { computed } from 'vue'

const { page } = useData()

const items = computed(() => {
  const path = page.value.relativePath
  const parts = path.split('/').filter(Boolean)
  const lastPart = parts[parts.length - 1]?.replace(/\.md$/, '') || ''
  const isChinesePage = path.startsWith('zh-CN/')

  const breadcrumbs = [
    {
      title: isChinesePage ? '首页' : 'Home',
      path: isChinesePage ? '/zh-CN/' : '/',
    },
  ]
  let currentPath = ''

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i]
    currentPath += `/${part}`
    if (part === 'zh-CN') continue

    breadcrumbs.push({
      title: part.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      path: currentPath + '/',
    })
  }

  if (lastPart && lastPart !== 'index') {
    breadcrumbs.push({
      title: lastPart.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      path: '/' + path.replace(/\.md$/, ''),
    })
  }
  return breadcrumbs
})
</script>

<template>
  <nav class="breadcrumb" aria-label="breadcrumb">
    <ol>
      <li v-for="(item, index) in items" :key="item.path">
        <a v-if="index < items.length - 1" :href="withBase(item.path)" class="breadcrumb-link">{{ item.title }}</a>
        <span v-else class="breadcrumb-current" aria-current="page">{{ item.title }}</span>
        <span v-if="index < items.length - 1" class="breadcrumb-separator">/</span>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.breadcrumb {
  padding: 1rem 0;
  font-size: 0.875rem;
}
.breadcrumb ol {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  list-style: none;
  margin: 0;
  padding: 0;
  gap: 0.25rem;
}
.breadcrumb li {
  display: flex;
  align-items: center;
}
.breadcrumb-link {
  color: var(--vp-c-text-2);
  text-decoration: none;
  transition: color 0.2s ease;
}
.breadcrumb-link:hover {
  color: var(--vp-c-brand-1);
}
.breadcrumb-current {
  color: var(--vp-c-text-1);
  font-weight: 500;
}
.breadcrumb-separator {
  margin: 0 0.25rem;
  color: var(--vp-c-divider);
  user-select: none;
}
</style>
