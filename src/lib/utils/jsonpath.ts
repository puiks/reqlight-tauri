/**
 * Extract a value from a parsed JSON object using a simple JSONPath expression.
 * Supports: $.foo.bar, $.items[0].name, $.data[2].nested.key
 * Extended: $[*] wildcard, $..key recursive descent
 * Returns the stringified value, or undefined if the path doesn't resolve.
 */
export function extractByPath(obj: unknown, path: string): string | undefined {
  if (obj === null || obj === undefined) return undefined

  // Normalize: strip leading "$." or "$"
  let normalized = path.trim()
  if (normalized.startsWith('$..')) {
    // Recursive descent from root - handle specially
    normalized = normalized.slice(1) // keep ".." prefix
  } else if (normalized.startsWith('$.')) {
    normalized = normalized.slice(2)
  } else if (normalized.startsWith('$[')) {
    normalized = normalized.slice(1)
  } else if (normalized === '$') {
    return stringify(obj)
  } else if (normalized.startsWith('$')) {
    normalized = normalized.slice(1)
  }

  if (!normalized) return stringify(obj)

  // Check for recursive descent
  if (normalized.startsWith('..')) {
    const rest = normalized.slice(2)
    return resolveRecursive(obj, rest)
  }

  // Tokenize: split by "." but handle array indices like "items[0]"
  const tokens = tokenize(normalized)
  const result = resolveTokens(obj, tokens)
  return result === undefined ? undefined : stringifyResult(result)
}

/** Resolve a list of tokens against a value, returning raw result (may be array for wildcards). */
function resolveTokens(current: unknown, tokens: string[]): unknown | undefined {
  let value: unknown = current

  for (let i = 0; i < tokens.length; i++) {
    if (value === null || value === undefined) return undefined
    const token = tokens[i]

    // Wildcard: [*] or key[*]
    const wildcardMatch = token.match(/^([^[]*)\[\*\]$/)
    if (wildcardMatch) {
      const [, key] = wildcardMatch
      if (key) {
        if (typeof value !== 'object' || value === null) return undefined
        value = (value as Record<string, unknown>)[key]
      }
      if (value === null || value === undefined) return undefined

      // Collect all elements/values
      let items: unknown[]
      if (Array.isArray(value)) {
        items = value
      } else if (typeof value === 'object') {
        items = Object.values(value)
      } else {
        return undefined
      }

      // If there are remaining tokens, resolve each item against them
      const remaining = tokens.slice(i + 1)
      if (remaining.length === 0) {
        return items
      }

      const results: unknown[] = []
      for (const item of items) {
        const r = resolveTokens(item, remaining)
        if (r !== undefined) results.push(r)
      }
      return results.length > 0 ? results : undefined
    }

    // Normal array index: key[0]
    const arrayMatch = token.match(/^([^[]*)\[(\d+)\]$/)
    if (arrayMatch) {
      const [, key, indexStr] = arrayMatch
      if (key) {
        if (typeof value !== 'object' || value === null) return undefined
        value = (value as Record<string, unknown>)[key]
      }
      if (!Array.isArray(value)) return undefined
      const index = parseInt(indexStr, 10)
      value = value[index]
    } else {
      if (typeof value !== 'object' || value === null) return undefined
      value = (value as Record<string, unknown>)[token]
    }
  }

  return value
}

/** Handle recursive descent: find all values matching a key recursively. */
function resolveRecursive(obj: unknown, rest: string): string | undefined {
  // rest is like "price" or "book[0].title"
  const tokens = tokenize(rest)
  if (tokens.length === 0) return undefined

  const firstToken = tokens[0]
  const remaining = tokens.slice(1)

  // Check if first token is a simple key (no array index, no wildcard)
  // If remaining is empty, collect all values for that key recursively
  // If remaining has tokens, try to resolve from each match
  if (remaining.length === 0 && !firstToken.includes('[')) {
    // Simple recursive key search: $..key
    const results = collectByKey(obj, firstToken)
    if (results.length === 0) return undefined
    if (results.length === 1) return stringify(results[0])
    return JSON.stringify(results)
  }

  // Recursive descent then path continuation: $..book[0].title
  // Find all occurrences of first key, then resolve remaining
  const firstKey = firstToken.match(/^([^[]+)/)?.[1]
  if (!firstKey) return undefined

  const matches = collectByKey(obj, firstKey)
  if (matches.length === 0) return undefined

  // Re-resolve with the array part + remaining from each match
  const hasArrayPart = firstToken.includes('[')
  if (hasArrayPart) {
    // e.g. "book[0]" - extract array index
    const idxMatch = firstToken.match(/\[(\d+)\]$/)
    if (idxMatch) {
      const idx = parseInt(idxMatch[1], 10)
      for (const match of matches) {
        if (Array.isArray(match) && idx < match.length) {
          const resolved = resolveTokens(match[idx], remaining)
          if (resolved !== undefined) return stringifyResult(resolved)
        }
      }
    }
    return undefined
  }

  // If remaining tokens exist, resolve through each match
  for (const match of matches) {
    const resolved = resolveTokens(match, remaining)
    if (resolved !== undefined) return stringifyResult(resolved)
  }
  return undefined
}

/** Recursively collect all values for a given key from an object tree. */
function collectByKey(obj: unknown, key: string): unknown[] {
  const results: unknown[] = []

  function walk(node: unknown) {
    if (node === null || node === undefined || typeof node !== 'object') return

    if (Array.isArray(node)) {
      for (const item of node) {
        walk(item)
      }
    } else {
      const record = node as Record<string, unknown>
      if (key in record) {
        results.push(record[key])
      }
      for (const val of Object.values(record)) {
        walk(val)
      }
    }
  }

  walk(obj)
  return results
}

function tokenize(path: string): string[] {
  const tokens: string[] = []
  let current = ''
  let inBracket = false
  for (let i = 0; i < path.length; i++) {
    const ch = path[i]
    if (ch === '[') inBracket = true
    if (ch === ']') inBracket = false
    if (ch === '.' && !inBracket) {
      if (current) tokens.push(current)
      current = ''
    } else {
      current += ch
    }
  }
  if (current) tokens.push(current)
  return tokens
}

function stringify(value: unknown): string | undefined {
  if (value === undefined) return undefined
  if (value === null) return 'null'
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  return JSON.stringify(value)
}

function stringifyResult(value: unknown): string | undefined {
  if (value === undefined) return undefined
  if (Array.isArray(value)) return JSON.stringify(value)
  return stringify(value)
}
