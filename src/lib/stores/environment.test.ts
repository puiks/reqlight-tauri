import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'
import { environmentStore } from './environment.svelte'

describe('environmentStore', () => {
  beforeEach(() => {
    environmentStore.environments = []
    environmentStore.activeEnvironmentId = null
  })

  it('adds an environment', () => {
    const env = environmentStore.addEnvironment('Dev')
    expect(environmentStore.environments).toHaveLength(1)
    expect(env.name).toBe('Dev')
    expect(env.id).toBeTruthy()
    expect(env.variables).toHaveLength(1) // default empty pair
  })

  it('updates an environment', () => {
    const env = environmentStore.addEnvironment('Dev')
    environmentStore.updateEnvironment({ ...env, name: 'Production' })
    expect(environmentStore.environments[0].name).toBe('Production')
  })

  it('deletes an environment', () => {
    const env = environmentStore.addEnvironment('Dev')
    environmentStore.deleteEnvironment(env.id)
    expect(environmentStore.environments).toHaveLength(0)
  })

  it('clears activeEnvironmentId when active env is deleted', () => {
    const env = environmentStore.addEnvironment('Dev')
    environmentStore.setActiveEnvironment(env.id)
    expect(environmentStore.activeEnvironmentId).toBe(env.id)
    environmentStore.deleteEnvironment(env.id)
    expect(environmentStore.activeEnvironmentId).toBeNull()
  })

  it('does not clear activeEnvironmentId when other env is deleted', () => {
    const env1 = environmentStore.addEnvironment('Dev')
    const env2 = environmentStore.addEnvironment('Staging')
    environmentStore.setActiveEnvironment(env1.id)
    environmentStore.deleteEnvironment(env2.id)
    expect(environmentStore.activeEnvironmentId).toBe(env1.id)
  })

  it('returns activeEnvironment from getter', () => {
    const env = environmentStore.addEnvironment('Dev')
    environmentStore.setActiveEnvironment(env.id)
    expect(environmentStore.activeEnvironment?.id).toBe(env.id)
  })

  it('returns undefined when no active environment', () => {
    expect(environmentStore.activeEnvironment).toBeUndefined()
  })

  it('masks secret values', () => {
    const env = environmentStore.addEnvironment('Dev')
    environmentStore.updateEnvironment({
      ...env,
      variables: [
        { id: '1', key: 'TOKEN', value: 'secret123', isEnabled: true, isSecret: true },
        { id: '2', key: 'HOST', value: 'example.com', isEnabled: true, isSecret: false },
      ],
    })
    environmentStore.maskSecrets()
    const vars = environmentStore.environments[0].variables
    expect(vars[0].value).toBe('••••••••')
    expect(vars[1].value).toBe('example.com')
  })

  it('fires onStateChange callback', () => {
    const cb = vi.fn()
    environmentStore.onStateChange(cb)
    environmentStore.addEnvironment('Dev')
    expect(cb).toHaveBeenCalledOnce()
  })
})
