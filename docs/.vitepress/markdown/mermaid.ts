import type MarkdownIt from 'markdown-it'
import { renderMermaidSVG } from 'beautiful-mermaid'

/**
 * Renders every ```mermaid fence to an inline SVG at build time.
 *
 * beautiful-mermaid writes the colours it is given as CSS custom properties on
 * the <svg> element. Pointing them at VitePress's theme variables makes the
 * diagrams follow light and dark mode with no JavaScript in the browser.
 *
 * Each SVG also carries a <style> block that is the same for every diagram:
 * a Google Fonts import and the rules that derive the drawing colours from
 * `--bg` and `--fg`. Vue does not allow <style> inside a page template, and
 * one copy is enough, so the block is removed here and its rules live in
 * `theme/diagram.css`.
 *
 * A diagram that does not parse fails the build, with the file name in the error.
 */
const palette = {
  bg: 'var(--vp-c-bg)',
  fg: 'var(--vp-c-text-1)',
  line: 'var(--vp-c-text-3)',
  muted: 'var(--vp-c-text-2)',
  surface: 'var(--vp-c-bg-soft)',
  border: 'var(--vp-c-divider)',
  accent: 'var(--vp-c-brand-1)',
}

const STYLE_BLOCK = /<style[^>]*>[\s\S]*?<\/style>/

export function mermaid(md: MarkdownIt): void {
  const fence = md.renderer.rules.fence!

  md.renderer.rules.fence = (tokens, idx, options, env, self) => {
    const token = tokens[idx]
    if (token.info.trim() !== 'mermaid') {
      return fence(tokens, idx, options, env, self)
    }

    let svg: string
    try {
      svg = renderMermaidSVG(token.content, { ...palette, transparent: true })
    } catch (error) {
      const file = env?.relativePath ?? 'unknown file'
      throw new Error(`Mermaid diagram in ${file} does not render: ${(error as Error).message}`)
    }
    return `<figure class="diagram">${svg.replace(STYLE_BLOCK, '')}</figure>\n`
  }
}
