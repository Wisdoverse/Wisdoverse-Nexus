import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Nexis',
  description: 'Enterprise-grade Real-time Collaboration Platform — 100K+ concurrent connections, cloud-native, AI-powered',
  
  // Multi-language routing
  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
    },
    'zh-CN': {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh-CN/',
    }
  },
  
  ignoreDeadLinks: true,
  
  head: [
    // Favicon and theme
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/images/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#6366f1' }],

    // SEO meta tags
    ['meta', { name: 'keywords', content: 'Nexis, real-time collaboration, WebSocket, Rust, cloud-native, AI, enterprise messaging' }],
    ['meta', { name: 'author', content: 'G-Brothers Group' }],
    ['meta', { name: 'description', content: 'Nexis is an enterprise-grade real-time collaboration platform built with Rust and Tokio. Supports 100K+ concurrent WebSocket connections with AI-powered summaries, plugin system, and multi-tenancy.' }],

    // Open Graph
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Nexis - Enterprise Real-time Collaboration Platform' }],
    ['meta', { property: 'og:description', content: '100K+ concurrent connections · AI-powered summaries · Plugin system · Multi-tenancy' }],
    ['meta', { property: 'og:image', content: 'https://gbrothersgroup.github.io/Nexis/images/og-image.png' }],
    ['meta', { property: 'og:url', content: 'https://gbrothersgroup.github.io/Nexis/' }],
    ['meta', { property: 'og:site_name', content: 'Nexis' }],

    // Twitter Card
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'Nexis - Enterprise Real-time Collaboration Platform' }],
    ['meta', { name: 'twitter:description', content: '100K+ concurrent connections · AI-powered summaries · Plugin system · Multi-tenancy' }],
    ['meta', { name: 'twitter:image', content: 'https://gbrothersgroup.github.io/Nexis/images/og-image.png' }],

    // Canonical URL
    ['link', { rel: 'canonical', href: 'https://gbrothersgroup.github.io/Nexis/' }],

    // Google Analytics (replace G-XXXXXXXXXX with your actual ID)
    // ['script', { async: '', src: 'https://www.googletagmanager.com/gtag/js?id=G-XXXXXXXXXX' }],
    // ['script', {}, `window.dataLayer = window.dataLayer || [];
    // function gtag(){dataLayer.push(arguments);}
    // gtag('js', new Date());
    // gtag('config', 'G-XXXXXXXXXX');`],
  ],
  
  themeConfig: {
    logo: '/images/logo.svg',
    siteTitle: 'Nexis',
    
    icon: {
      type: 'svg'
    },
    
    // Breadcrumb navigation
    breadcrumb: true,
    
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Quick Start', link: '/en/getting-started/quick-start' },
      { text: 'Docs', link: '/en/architecture/' },
      {
        text: 'Resources',
        items: [
          { text: 'Roadmap', link: '/en/roadmap' },
          { text: 'Contributing', link: '/en/development/contributing' },
          { text: 'Security', link: '/en/security/audit-report' },
          { text: 'Performance', link: '/en/performance/benchmark-report' },
        ]
      }
    ],
    
    sidebar: {
      '/en/': [
        {
          text: '📚 Introduction',
          items: [
            { text: 'Overview', link: '/en/' }
          ]
        },
        {
          text: '🚀 Getting Started',
          collapsed: false,
          items: [
            { text: 'Quick Start (5 min)', link: '/en/getting-started/quick-start' },
            { text: 'Development Guide', link: '/en/getting-started/development-guide' }
          ]
        },
        {
          text: '🏗️ Architecture',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/en/architecture/' },
            { text: 'Data Flow', link: '/en/architecture/data-flow' },
            { text: 'Components', link: '/en/architecture/components' }
          ]
        },
        {
          text: '📡 API Reference',
          collapsed: true,
          items: [
            { text: 'Metrics API', link: '/en/api/metrics' },
            { text: 'Versioning', link: '/en/api/versioning' }
          ]
        },
        {
          text: '🔍 Observability',
          collapsed: true,
          items: [
            { text: 'Tracing', link: '/en/observability/tracing' }
          ]
        },
        {
          text: '⚡ Performance',
          collapsed: true,
          items: [
            { text: 'Benchmark Report', link: '/en/performance/benchmark-report' }
          ]
        },
        {
          text: '🛡️ Security',
          collapsed: true,
          items: [
            { text: 'Audit Report', link: '/en/security/audit-report' }
          ]
        },
        {
          text: '🤝 Community',
          collapsed: true,
          items: [
            { text: 'Contributing Guide', link: '/en/development/contributing' },
            { text: 'Roadmap', link: '/en/roadmap' }
          ]
        }
      ],
      '/zh-CN/': [
        {
          text: '📚 介绍',
          items: [
            { text: '概述', link: '/zh-CN/' }
          ]
        },
        {
          text: '🚀 快速开始',
          collapsed: false,
          items: [
            { text: '5 分钟入门', link: '/zh-CN/getting-started/quick-start' },
            { text: '开发指南', link: '/zh-CN/getting-started/development-guide' }
          ]
        },
        {
          text: '🏗️ 架构',
          collapsed: false,
          items: [
            { text: '概述', link: '/zh-CN/architecture/' },
            { text: '数据流', link: '/zh-CN/architecture/data-flow' },
            { text: '组件', link: '/zh-CN/architecture/components' }
          ]
        },
        {
          text: '📡 API 参考',
          collapsed: true,
          items: [
            { text: 'Metrics API', link: '/zh-CN/api/metrics' },
            { text: '版本控制', link: '/zh-CN/api/versioning' }
          ]
        },
        {
          text: '🔍 可观测性',
          collapsed: true,
          items: [
            { text: '链路追踪', link: '/zh-CN/observability/tracing' }
          ]
        },
        {
          text: '⚡ 性能',
          collapsed: true,
          items: [
            { text: '基准测试报告', link: '/zh-CN/performance/benchmark-report' }
          ]
        },
        {
          text: '🛡️ 安全',
          collapsed: true,
          items: [
            { text: '安全审计报告', link: '/zh-CN/security/audit-report' }
          ]
        },
        {
          text: '🤝 社区',
          collapsed: true,
          items: [
            { text: '贡献指南', link: '/zh-CN/development/contributing' },
            { text: '路线图', link: '/zh-CN/roadmap' }
          ]
        }
      ]
    },
    
    socialLinks: [
      { icon: 'github', link: 'https://github.com/gbrothersgroup/Nexis' },
      { icon: 'discord', link: 'https://discord.gg/clawd' }
    ],
    
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026-present G-Brothers Group'
    },
    
    search: {
      provider: 'local',
      options: {
        translations: {
          button: {
            buttonText: 'Search Documentation',
            buttonAriaLabel: 'Search'
          }
        }
      }
    },
    
    editLink: {
      pattern: 'https://github.com/gbrothersgroup/Nexis/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    },
    
    outline: {
      level: [2, 3],
      label: 'On this page'
    },
    
    lastUpdated: {
      text: 'Last updated',
      formatOptions: {
        dateStyle: 'medium',
        timeStyle: 'short'
      }
    },
    
    docFooter: {
      prev: 'Previous Page',
      next: 'Next Page'
    }
  }
})
