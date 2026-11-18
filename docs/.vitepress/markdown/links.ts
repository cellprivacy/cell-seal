import type MarkdownIt from 'markdown-it'

/**
 * The documents link to each other the way GitHub expects. Four kinds of link
 * need a different target on the published site:
 *
 *   ../../README.md     the repository README         ->  the product's landing page
 *   <folder>/README.md  a folder index                ->  <folder>/index.md
 *   seal/architecture.md  published under a new name  ->  seal/system-architecture.md
 *   screens/fund/       a folder of PNG exports       ->  the folder on GitHub
 *
 * VitePress resolves the result against the rewritten page paths, so the
 * replacements are written as source file names, not as URLs.
 */
const GITHUB_DOCS = 'https://github.com/cellprivacy/cell-seal/tree/main/docs/'

function target(href: string, page: string): string {
  if (href === '../../README.md') return './index.md'
  if (href.endsWith('README.md')) return href.replace(/README\.md$/, 'index.md')
  if (page.startsWith('seal/') && /(^|\/)architecture\.md(#|$)/.test(href)) {
    return href.replace('architecture.md', 'system-architecture.md')
  }
  if (/^screens\/(fund|investor)\/$/.test(href)) return GITHUB_DOCS + 'design/' + href
  return href
}

export function repositoryLinks(md: MarkdownIt): void {
  md.core.ruler.push('repository_links', (state) => {
    const page: string = state.env?.relativePath ?? ''
    for (const block of state.tokens) {
      if (block.type !== 'inline' || !block.children) continue
      for (const token of block.children) {
        if (token.type !== 'link_open') continue
        const href = token.attrGet('href')
        if (href) token.attrSet('href', target(href, page))
      }
    }
  })
}
