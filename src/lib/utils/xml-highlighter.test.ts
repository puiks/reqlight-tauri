import { describe, expect, it } from 'vite-plus/test'
import { formatXml, highlightXml } from './xml-highlighter'

describe('highlightXml', () => {
  it('highlights a simple tag', () => {
    const result = highlightXml('<div>')
    expect(result).toContain('class="xml-bracket"')
    expect(result).toContain('class="xml-tag"')
    expect(result).toContain('div')
  })

  it('highlights closing tags', () => {
    const result = highlightXml('</div>')
    expect(result).toContain('class="xml-bracket"')
    expect(result).toContain('class="xml-tag"')
  })

  it('highlights self-closing tags', () => {
    const result = highlightXml('<br/>')
    expect(result).toContain('class="xml-tag"')
    expect(result).toContain('class="xml-bracket"')
  })

  it('highlights attributes and values', () => {
    const result = highlightXml('<div class="main">')
    expect(result).toContain('class="xml-attr"')
    expect(result).toContain('class="xml-value"')
    expect(result).toContain('main')
  })

  it('highlights XML comments', () => {
    const result = highlightXml('<!-- comment -->')
    expect(result).toContain('class="xml-comment"')
    expect(result).toContain('comment')
  })

  it('escapes text content', () => {
    const result = highlightXml('<p>a &amp; b</p>')
    expect(result).toContain('a &amp;amp; b')
  })

  it('handles mixed content', () => {
    const result = highlightXml('<root><child attr="val">text</child></root>')
    expect(result).toContain('class="xml-tag"')
    expect(result).toContain('class="xml-attr"')
    expect(result).toContain('text')
  })
})

describe('formatXml', () => {
  it('indents nested tags', () => {
    const input = '<root><child>text</child></root>'
    const result = formatXml(input)
    const lines = result.split('\n')
    expect(lines[0]).toBe('<root>')
    expect(lines[1]).toBe('  <child>text</child>')
    expect(lines[2]).toBe('</root>')
  })

  it('handles self-closing tags', () => {
    const input = '<root><br/></root>'
    const result = formatXml(input)
    expect(result).toContain('  <br/>')
  })

  it('handles processing instructions', () => {
    const input = '<?xml version="1.0"?><root/>'
    const result = formatXml(input)
    expect(result.split('\n')[0]).toContain('<?xml')
  })

  it('handles empty input', () => {
    expect(formatXml('')).toBe('')
  })

  it('preserves content within tags', () => {
    const input = '<name>John</name>'
    const result = formatXml(input)
    expect(result).toContain('John')
  })
})
