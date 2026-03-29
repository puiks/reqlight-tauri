import { describe, expect, it } from 'vitest'
import { extractVariableNames, collectRequestVariables, findUnmatchedVariables } from './variables'
import type { SavedRequest, KeyValuePair } from '../types'

function makePair(key: string, value: string): KeyValuePair {
  return { id: crypto.randomUUID(), key, value, isEnabled: true, isSecret: false }
}

const baseRequest: SavedRequest = {
  id: '1',
  name: 'test',
  method: 'GET',
  url: 'https://{{host}}/api',
  headers: [],
  queryParams: [],
  body: 'none',
  auth: 'none',
  assertions: [],
  sortOrder: 0,
  createdAt: '2025-01-01T00:00:00Z',
  updatedAt: '2025-01-01T00:00:00Z',
}

describe('extractVariableNames', () => {
  it('extracts variable names from a string', () => {
    expect(extractVariableNames('Hello {{name}}, your id is {{id}}')).toEqual(['name', 'id'])
  })

  it('returns empty array for no variables', () => {
    expect(extractVariableNames('plain text')).toEqual([])
  })

  it('deduplicates variable names', () => {
    expect(extractVariableNames('{{x}} and {{x}}')).toEqual(['x'])
  })

  it('trims whitespace inside braces', () => {
    expect(extractVariableNames('{{ host }}')).toEqual(['host'])
  })

  it('handles empty braces', () => {
    expect(extractVariableNames('{{}}')).toEqual([])
  })
})

describe('collectRequestVariables', () => {
  it('extracts from URL', () => {
    expect(collectRequestVariables(baseRequest)).toEqual(['host'])
  })

  it('extracts from headers', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      headers: [makePair('Authorization', 'Bearer {{token}}')],
    }
    expect(collectRequestVariables(req)).toEqual(['token'])
  })

  it('ignores disabled headers', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      headers: [{ ...makePair('X-Key', '{{secret}}'), isEnabled: false }],
    }
    expect(collectRequestVariables(req)).toEqual([])
  })

  it('extracts from JSON body', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      body: { json: '{"user": "{{username}}"}' },
    }
    expect(collectRequestVariables(req)).toEqual(['username'])
  })

  it('extracts from bearer auth', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      auth: { bearerToken: { token: '{{api_token}}' } },
    }
    expect(collectRequestVariables(req)).toEqual(['api_token'])
  })

  it('extracts from form data body', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      body: { formData: [makePair('field', '{{val}}')] },
    }
    expect(collectRequestVariables(req)).toEqual(['val'])
  })
})

describe('findUnmatchedVariables', () => {
  const env: KeyValuePair[] = [makePair('host', 'api.example.com'), makePair('token', 'abc123')]

  it('returns empty when all variables are matched', () => {
    const req: SavedRequest = {
      ...baseRequest,
      headers: [makePair('Authorization', 'Bearer {{token}}')],
    }
    expect(findUnmatchedVariables(req, env)).toEqual([])
  })

  it('returns unmatched variable names', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://{{host}}/{{version}}/api',
    }
    expect(findUnmatchedVariables(req, env)).toEqual(['version'])
  })

  it('ignores disabled environment variables', () => {
    const disabledEnv: KeyValuePair[] = [{ ...env[0], isEnabled: false }, env[1]]
    expect(findUnmatchedVariables(baseRequest, disabledEnv)).toEqual(['host'])
  })

  it('returns empty when no variables used', () => {
    const req: SavedRequest = {
      ...baseRequest,
      url: 'https://example.com',
      headers: [],
    }
    expect(findUnmatchedVariables(req, env)).toEqual([])
  })
})
