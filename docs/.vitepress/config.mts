import { defineConfig } from 'vitepress'
import { cpSync, existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { sidebar } from './sidebar'
import { mermaid } from './markdown/mermaid'
import { repositoryLinks } from './markdown/links'

/**
 * One folder per product, one URL prefix per product:
 *
 *   docs/seal/     ->  docs.cellprivate.com/seal/
 *   docs/channel/  ->  docs.cellprivate.com/channel/   (when it exists)
 *
 * Inside a product folder the file name is the URL, with three exceptions:
 *
 *   README.md                          ->  index.md            GitHub folder index
 *   seal/technical-architecture.md     ->  seal/architecture   the SCF document
 *   seal/architecture.md               ->  seal/system-architecture
 */
function rewrite(source: string): string {
  if (source === 'seal/technical-architecture.md') return 'seal/architecture.md'
  if (source === 'seal/architecture.md') return 'seal/system-architecture.md'
  return source.replace(/(^|\/)README\.md$/, '$1index.md')
}

/** Files a product links to that are not markdown or images: copied as they are. */
const STATIC = [['seal/design/screens', 'seal/design/screens'], ['seal/design/cell-seal.pen', 'seal/design/cell-seal.pen']]

export default defineConfig({
  title: 'Cell Private',
  description: 'Technical documentation for Cell Private products on Stellar',
  lang: 'en',
  cleanUrls: true,
  rewrites: rewrite,
  sitemap: { hostname: 'https://docs.cellprivate.com' },
  // The Pen source file is copied by `buildEnd`, so links to it are valid.
  ignoreDeadLinks: [/\.pen$/],
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#0a0b0d' }],
  ],

  markdown: {
    config(md) {
      md.use(mermaid)
      md.use(repositoryLinks)
    },
  },

  themeConfig: {
    logo: '/logo.svg',
    // One entry per product. Add { text: 'Channel', link: '/channel/', activeMatch: '^/channel/' } later.
    nav: [{ text: 'Seal', link: '/seal/', activeMatch: '^/seal/' }],
    sidebar,
    // Section headings only. Every document numbers its sections, so the
    // outline reads as a table of contents without the subsection noise.
    outline: { level: 2, label: 'Contents' },
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: 'https://github.com/cellprivacy/cell-seal' }],
    footer: { message: 'Cell Private technical documentation.', copyright: 'Cell Private' },
  },

  buildEnd({ srcDir, outDir }) {
    for (const [from, to] of STATIC) {
      const src = resolve(srcDir, from)
      if (existsSync(src)) cpSync(src, resolve(outDir, to), { recursive: true })
    }
  },
})
