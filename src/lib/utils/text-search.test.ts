import { describe, expect, it } from 'vite-plus/test'
import { findMatches, highlightMatches } from './text-search'

describe('findMatches', () => {
  it('returns empty for empty query', () => {
    expect(findMatches('hello world', '')).toEqual([])
  })

  it('finds all occurrences case-insensitively', () => {
    const result = findMatches('Hello hello HELLO', 'hello')
    expect(result).toEqual([
      { start: 0, end: 5 },
      { start: 6, end: 11 },
      { start: 12, end: 17 },
    ])
  })

  it('finds case-sensitive matches', () => {
    const result = findMatches('Hello hello HELLO', 'hello', true)
    expect(result).toEqual([{ start: 6, end: 11 }])
  })

  it('finds overlapping positions correctly', () => {
    const result = findMatches('aaa', 'aa')
    expect(result).toEqual([
      { start: 0, end: 2 },
      { start: 1, end: 3 },
    ])
  })

  it('returns empty when no match', () => {
    expect(findMatches('hello', 'xyz')).toEqual([])
  })
})

describe('highlightMatches', () => {
  it('returns escaped text when no matches', () => {
    expect(highlightMatches('<b>test</b>', [])).toBe('&lt;b&gt;test&lt;/b&gt;')
  })

  it('wraps matches with mark tags', () => {
    const matches = [{ start: 0, end: 5 }]
    const result = highlightMatches('hello world', matches)
    expect(result).toBe('<mark class="search-match">hello</mark> world')
  })

  it('marks active match', () => {
    const matches = [
      { start: 0, end: 2 },
      { start: 6, end: 8 },
    ]
    const result = highlightMatches('hi xxx hi', matches, 1)
    expect(result).toContain('class="search-match active"')
  })

  it('escapes HTML in matched text', () => {
    const matches = [{ start: 0, end: 3 }]
    const result = highlightMatches('<b>', matches)
    expect(result).toBe('<mark class="search-match">&lt;b&gt;</mark>')
  })
})
