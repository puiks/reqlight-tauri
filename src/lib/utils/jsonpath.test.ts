import { describe, expect, it } from 'vite-plus/test'
import { extractByPath } from './jsonpath'

describe('extractByPath', () => {
  const sample = {
    data: {
      token: 'abc123',
      user: { name: 'Alice', age: 30 },
      items: [
        { id: 1, title: 'First' },
        { id: 2, title: 'Second' },
      ],
    },
    status: 'ok',
    count: 42,
    active: true,
  }

  it('extracts simple dot notation', () => {
    expect(extractByPath(sample, '$.data.token')).toBe('abc123')
  })

  it('extracts nested dot notation', () => {
    expect(extractByPath(sample, '$.data.user.name')).toBe('Alice')
  })

  it('extracts number value as string', () => {
    expect(extractByPath(sample, '$.data.user.age')).toBe('30')
  })

  it('extracts boolean value as string', () => {
    expect(extractByPath(sample, '$.active')).toBe('true')
  })

  it('extracts root-level key', () => {
    expect(extractByPath(sample, '$.status')).toBe('ok')
  })

  it('extracts number at root', () => {
    expect(extractByPath(sample, '$.count')).toBe('42')
  })

  it('returns entire object for $', () => {
    const result = extractByPath(sample, '$')
    expect(result).toBeDefined()
    expect(JSON.parse(result!)).toEqual(sample)
  })

  it('extracts array element by index', () => {
    expect(extractByPath(sample, '$.data.items[0].title')).toBe('First')
    expect(extractByPath(sample, '$.data.items[1].id')).toBe('2')
  })

  it('extracts array element at root index', () => {
    const arr = [{ x: 10 }, { x: 20 }]
    expect(extractByPath(arr, '$[0].x')).toBe('10')
    expect(extractByPath(arr, '$[1].x')).toBe('20')
  })

  it('returns undefined for missing path', () => {
    expect(extractByPath(sample, '$.nonexistent')).toBeUndefined()
    expect(extractByPath(sample, '$.data.missing.deep')).toBeUndefined()
  })

  it('returns undefined for out-of-bounds array index', () => {
    expect(extractByPath(sample, '$.data.items[99].title')).toBeUndefined()
  })

  it('returns undefined for null input', () => {
    expect(extractByPath(null, '$.foo')).toBeUndefined()
  })

  it('returns undefined for undefined input', () => {
    expect(extractByPath(undefined, '$.foo')).toBeUndefined()
  })

  it('returns undefined when accessing property on non-object', () => {
    expect(extractByPath(sample, '$.status.length')).toBeUndefined()
  })

  it('returns JSON string for object value', () => {
    const result = extractByPath(sample, '$.data.user')
    expect(result).toBe('{"name":"Alice","age":30}')
  })

  it('returns JSON string for array value', () => {
    const result = extractByPath(sample, '$.data.items')
    expect(result).toBeDefined()
    expect(JSON.parse(result!)).toEqual(sample.data.items)
  })

  it('handles path without $. prefix', () => {
    expect(extractByPath(sample, 'data.token')).toBe('abc123')
  })

  it('returns null as string for null value', () => {
    const obj = { val: null }
    expect(extractByPath(obj, '$.val')).toBe('null')
  })
})

describe('extractByPath - wildcard [*]', () => {
  it('collects all elements of an array', () => {
    const data = { items: [1, 2, 3] }
    expect(extractByPath(data, '$.items[*]')).toBe('[1,2,3]')
  })

  it('collects a field from all array elements', () => {
    const data = {
      users: [{ name: 'Alice' }, { name: 'Bob' }, { name: 'Charlie' }],
    }
    const result = extractByPath(data, '$.users[*].name')
    expect(result).toBe('["Alice","Bob","Charlie"]')
  })

  it('collects all values of an object via wildcard', () => {
    const data = { obj: { a: 1, b: 2, c: 3 } }
    const result = extractByPath(data, '$.obj[*]')
    const parsed = JSON.parse(result!)
    expect(parsed).toEqual(expect.arrayContaining([1, 2, 3]))
    expect(parsed).toHaveLength(3)
  })

  it('returns undefined when wildcard on non-iterable', () => {
    const data = { val: 42 }
    expect(extractByPath(data, '$.val[*]')).toBeUndefined()
  })
})

describe('extractByPath - recursive descent (..)', () => {
  const nested = {
    store: {
      book: [
        { title: 'A', price: 8.95 },
        { title: 'B', price: 12.99 },
      ],
      name: 'My Store',
    },
    metadata: { price: 0 },
  }

  it('finds all matching keys recursively', () => {
    const result = extractByPath(nested, '$..price')
    const parsed = JSON.parse(result!)
    expect(parsed).toEqual(expect.arrayContaining([8.95, 12.99, 0]))
    expect(parsed).toHaveLength(3)
  })

  it('finds nested key across levels', () => {
    const data = { a: { name: 'first' }, b: { c: { name: 'second' } } }
    const result = extractByPath(data, '$..name')
    const parsed = JSON.parse(result!)
    expect(parsed).toEqual(expect.arrayContaining(['first', 'second']))
    expect(parsed).toHaveLength(2)
  })

  it('recursive descent followed by array index', () => {
    const result = extractByPath(nested, '$..book[0].title')
    expect(result).toBe('A')
  })

  it('returns undefined when recursive key not found', () => {
    expect(extractByPath(nested, '$..nonexistent')).toBeUndefined()
  })

  it('recursive descent collects from arrays too', () => {
    const data = { items: [{ id: 1 }, { id: 2 }] }
    const result = extractByPath(data, '$..id')
    const parsed = JSON.parse(result!)
    expect(parsed).toEqual([1, 2])
  })
})
