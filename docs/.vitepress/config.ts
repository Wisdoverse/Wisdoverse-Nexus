import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Wisdoverse Nexus',
  description:
    'Source-available AI-native collaboration infrastructure for real-time rooms, identity, WebSocket messaging, and extensible workflows.',

  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
    },
    'zh-CN': {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh-CN/',
    },
  },

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/images/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#2563eb' }],
    [
      'meta',
      {
        name: 'keywords',
        content:
          'Wisdoverse Nexus, source available collaboration, WebSocket, Rust, AI collaboration, real-time messaging',
      },
    ],
    ['meta', { name: 'author', content: 'Wisdoverse' }],
    [
      'meta',
      {
        name: 'description',
        content:
          'Wisdoverse Nexus is a source-available Rust-first collaboration platform for real-time messaging, shared identity, AI provider integration, and extensible workflows.',
      },
    ],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Wisdoverse Nexus' }],
    [
      'meta',
      {
        property: 'og:description',
        content:
          'Source-available AI-native collaboration infrastructure for real-time rooms, identity, WebSocket messaging, and extensible workflows.',
      },
    ],
    ['meta', { property: 'og:image', content: 'https://wisdoverse.com/images/og-image.png' }],
    ['meta', { property: 'og:url', content: 'https://wisdoverse.com/' }],
    ['meta', { property: 'og:site_name', content: 'Wisdoverse Nexus' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'Wisdoverse Nexus' }],
    [
      'meta',
      {
        name: 'twitter:description',
        content:
          'Source-available AI-native collaboration infrastructure for real-time rooms, identity, WebSocket messaging, and extensible workflows.',
      },
    ],
    ['meta', { name: 'twitter:image', content: 'https://wisdoverse.com/images/og-image.png' }],
    ['link', { rel: 'canonical', href: 'https://wisdoverse.com/' }],
  ],

  themeConfig: {
    logo: '/images/logo.svg',
    siteTitle: 'Wisdoverse Nexus',

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Quick Start', link: '/en/getting-started/quick-start' },
      { text: 'Architecture', link: '/en/architecture' },
      { text: 'API', link: '/en/api/reference' },
      {
        text: 'Community',
        items: [
          { text: 'Contributing', link: '/en/development/contributing' },
          { text: 'Roadmap', link: '/en/roadmap' },
          { text: 'Security', link: '/en/architecture/security/overview' },
          { text: 'GitHub', link: 'https://github.com/Wisdoverse/Wisdoverse-Nexus' },
        ],
      },
    ],

    sidebar: {
      '/en/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Quick Start', link: '/en/getting-started/quick-start' },
            { text: 'Development Guide', link: '/en/getting-started/development-guide' },
            { text: 'Deployment Guide', link: '/en/getting-started/deployment-guide' },
          ],
        },
        {
          text: 'Architecture',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/en/architecture' },
            { text: 'Tenant Model', link: '/en/architecture/tenant-model' },
            { text: 'Security Overview', link: '/en/architecture/security/overview' },
            { text: 'ADRs', link: '/en/architecture/adr/' },
          ],
        },
        {
          text: 'API',
          collapsed: true,
          items: [
            { text: 'Reference', link: '/en/api/reference' },
            { text: 'WebSocket', link: '/en/api/websocket' },
            { text: 'Metrics', link: '/en/api/metrics' },
            { text: 'Versioning', link: '/en/api/versioning' },
          ],
        },
        {
          text: 'Operations',
          collapsed: true,
          items: [
            { text: 'Deployment', link: '/en/operations/deployment' },
            { text: 'Monitoring', link: '/en/operations/monitoring' },
            { text: 'Troubleshooting', link: '/en/operations/troubleshooting' },
            { text: 'Tracing', link: '/en/observability/tracing' },
          ],
        },
        {
          text: 'Project',
          collapsed: true,
          items: [
            { text: 'Contributing', link: '/en/development/contributing' },
            { text: 'Testing', link: '/en/guides/testing' },
            { text: 'Release Process', link: '/en/guides/release-process' },
            { text: 'Roadmap', link: '/en/roadmap' },
            { text: 'License', link: '/en/license' },
            { text: 'Performance', link: '/en/performance/benchmark-report' },
          ],
        },
      ],
      '/zh-CN/': [
        {
          text: '开始',
          items: [
            { text: '快速开始', link: '/zh-CN/getting-started/quick-start' },
            { text: '开发指南', link: '/zh-CN/getting-started/development-guide' },
          ],
        },
        {
          text: '架构与项目',
          collapsed: false,
          items: [
            { text: '架构概览', link: '/zh-CN/architecture/' },
            { text: '贡献指南', link: '/zh-CN/development/contributing' },
            { text: '路线图', link: '/zh-CN/roadmap' },
          ],
        },
        {
          text: '质量与安全',
          collapsed: true,
          items: [
            { text: '安全概览', link: '/zh-CN/security/overview' },
            { text: '性能报告', link: '/zh-CN/performance/benchmark-report' },
          ],
        },
      ],
    },

    socialLinks: [{ icon: 'github', link: 'https://github.com/Wisdoverse/Wisdoverse-Nexus' }],

    footer: {
      message: 'Source-available under the Wisdoverse Nexus Business Source License 1.1.',
      copyright: 'Copyright 2026-present Wisdoverse',
    },

    search: {
      provider: 'local',
      options: {
        translations: {
          button: {
            buttonText: 'Search Documentation',
            buttonAriaLabel: 'Search',
          },
        },
      },
    },

    editLink: {
      pattern: 'https://github.com/Wisdoverse/Wisdoverse-Nexus/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },

    outline: {
      level: [2, 3],
      label: 'On this page',
    },

    lastUpdated: {
      text: 'Last updated',
      formatOptions: {
        dateStyle: 'medium',
        timeStyle: 'short',
      },
    },

    docFooter: {
      prev: 'Previous Page',
      next: 'Next Page',
    },
  },
})
