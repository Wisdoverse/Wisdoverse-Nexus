// Custom theme extension for Wisdoverse Nexus
import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'

// Import custom CSS for animations and enhancements
import './custom.css'

export default {
  extends: DefaultTheme,
  
  // Setup for page transition animations
  setup() {
    if (typeof window !== 'undefined') {
      // Add fade-in animation class to document
      document.addEventListener('DOMContentLoaded', () => {
        document.body.classList.add('page-loaded')
      })
    }
  }
} satisfies Theme
