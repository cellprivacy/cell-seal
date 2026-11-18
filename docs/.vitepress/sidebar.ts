import type { DefaultTheme } from 'vitepress'

/**
 * One key per product prefix. A second product adds its own key, for example
 * '/channel/': [...], and VitePress shows the sidebar that matches the URL.
 *
 * Seal: four groups, in reading order. The first holds the documents everyone reads;
 * the others follow the Scope column of the repository README: what a platform
 * receives (Product), how it is built (Design), and the reference platform
 * that ships with it (Sandbox).
 */
export const sidebar: DefaultTheme.Sidebar = {
  '/seal/': [
    {
      text: 'Cell Seal',
      items: [
        { text: 'Technical Architecture', link: '/seal/architecture' },
        { text: 'Overview', link: '/seal/overview' },
        { text: 'Glossary', link: '/seal/glossary' },
      ],
    },
    {
      text: 'Product',
      items: [
        { text: 'Flows', link: '/seal/flows' },
        { text: 'Integration', link: '/seal/integration' },
        { text: 'Onboarding a fund', link: '/seal/onboarding' },
        { text: 'Primitives', link: '/seal/primitives' },
      ],
    },
    {
      text: 'Design',
      items: [
        { text: 'Architecture in detail', link: '/seal/system-architecture' },
        { text: 'Contracts', link: '/seal/contracts' },
        { text: 'Cryptography', link: '/seal/cryptography' },
        { text: 'Keys', link: '/seal/keys' },
        { text: 'SDK', link: '/seal/sdk' },
        { text: 'Security', link: '/seal/security' },
      ],
    },
    {
      text: 'Sandbox',
      items: [
        { text: 'Backend', link: '/seal/backend' },
        { text: 'Frontend', link: '/seal/frontend' },
        { text: 'Screens', link: '/seal/design/' },
      ],
    },
  ],
}
