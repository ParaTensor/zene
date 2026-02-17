import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "Zene",
  description: "A self-healing, multi-agent coding engine written in Rust.",

  appearance: 'dark', // Force dark mode for that hacker aesthetic

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    // Preload JetBrains Mono
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap', rel: 'stylesheet' }],
    // Meta tags for SEO and social sharing
    ['meta', { property: 'og:title', content: 'Zene - The Self-Healing AI Coding Agent' }],
    ['meta', { property: 'og:description', content: 'Plan, Execute, Reflect. An autonomous coding engine that verifies its own work.' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
  ],

  themeConfig: {
    logo: '/logo.svg', // Placeholder, you can add an image later
    siteTitle: 'Zene',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Blog', link: '/blog/' },
      { text: 'Examples', link: '/examples/' },
      {
        text: 'v0.2.2', items: [
          { text: 'Changelog', link: 'https://github.com/lipish/zene/blob/main/CHANGELOG.md' },
          { text: 'Contributing', link: 'https://github.com/lipish/zene/blob/main/CONTRIBUTING.md' }
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'What is Zene?', link: '/guide/what-is-zene' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Architecture', link: '/guide/architecture' }
          ]
        },
        {
          text: 'Core Concepts',
          items: [
            { text: 'Planner', link: '/guide/planner' },
            { text: 'Executor', link: '/guide/executor' },
            { text: 'Reflector', link: '/guide/reflector' },
            { text: 'Simple Mode', link: '/guide/simple-mode' }
          ]
        },
        {
          text: 'Extensibility',
          items: [
            { text: 'MCP Integration', link: '/guide/mcp' }
          ]
        }
      ],
      '/blog/': [
        {
          text: 'Case Studies',
          items: [
            { text: 'Python Env Verification', link: '/blog/python-env-verification' },
            { text: 'Data Analysis', link: '/blog/data-analysis' },
            { text: 'Multi-File API', link: '/blog/multi-file-api' },
            { text: 'Dockerization', link: '/blog/dockerization' },
            { text: 'Unit Testing', link: '/blog/unit-testing' },
            { text: 'Web Scraping', link: '/blog/web-scraping' },
            { text: 'The Self-Healing Compiler', link: '/blog/self-healing-compiler' },
            { text: 'Refactoring Legacy Code', link: '/blog/refactoring-legacy' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/lipish/zene' },
      { icon: 'twitter', link: 'https://twitter.com/lipish' } // Replace with your handle
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026 Zene Contributors'
    },

    search: {
      provider: 'local'
    }
  }
})
