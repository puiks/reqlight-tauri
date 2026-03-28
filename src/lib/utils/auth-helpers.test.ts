import { describe, expect, it } from 'vite-plus/test'
import type { AuthConfig } from '../types'
import { buildOAuth2ConfigFromFields, defaultOAuth2Fields, parseAuthConfig } from './auth-helpers'

describe('defaultOAuth2Fields', () => {
  it('returns empty/default values', () => {
    const fields = defaultOAuth2Fields()
    expect(fields.oauthGrantType).toBe('client_credentials')
    expect(fields.oauthClientId).toBe('')
    expect(fields.oauthClientSecret).toBe('')
    expect(fields.oauthAuthUrl).toBe('')
    expect(fields.oauthTokenUrl).toBe('')
    expect(fields.oauthScopes).toBe('')
    expect(fields.oauthAccessToken).toBe('')
    expect(fields.oauthRefreshToken).toBe('')
    expect(fields.oauthTokenExpiry).toBeNull()
  })
})

describe('parseAuthConfig', () => {
  it('returns none auth for undefined', () => {
    const fields = parseAuthConfig(undefined)
    expect(fields.authType).toBe('none')
    expect(fields.bearerToken).toBe('')
    expect(fields.basicUsername).toBe('')
  })

  it('parses bearer token auth', () => {
    const auth: AuthConfig = { bearerToken: { token: 'my-token' } }
    const fields = parseAuthConfig(auth)
    expect(fields.authType).toBe('bearerToken')
    expect(fields.bearerToken).toBe('my-token')
  })

  it('parses basic auth', () => {
    const auth: AuthConfig = { basicAuth: { username: 'user', password: 'pass' } }
    const fields = parseAuthConfig(auth)
    expect(fields.authType).toBe('basicAuth')
    expect(fields.basicUsername).toBe('user')
    expect(fields.basicPassword).toBe('pass')
  })

  it('parses api key auth', () => {
    const auth: AuthConfig = { apiKey: { key: 'X-Key', value: 'val', location: 'query' } }
    const fields = parseAuthConfig(auth)
    expect(fields.authType).toBe('apiKey')
    expect(fields.apiKeyKey).toBe('X-Key')
    expect(fields.apiKeyValue).toBe('val')
    expect(fields.apiKeyLocation).toBe('query')
  })

  it('parses oauth2 auth', () => {
    const auth: AuthConfig = {
      oauth2: {
        grantType: 'authorization_code',
        clientId: 'cid',
        clientSecret: 'csecret',
        authUrl: 'https://auth',
        tokenUrl: 'https://token',
        scopes: 'read write',
        accessToken: 'at',
        refreshToken: 'rt',
        tokenExpiry: '2025-01-01T00:00:00Z',
      },
    }
    const fields = parseAuthConfig(auth)
    expect(fields.authType).toBe('oauth2')
    expect(fields.oauthGrantType).toBe('authorization_code')
    expect(fields.oauthClientId).toBe('cid')
    expect(fields.oauthAccessToken).toBe('at')
    expect(fields.oauthTokenExpiry).toBe('2025-01-01T00:00:00Z')
  })
})

describe('buildOAuth2ConfigFromFields', () => {
  it('builds config from fields', () => {
    const config = buildOAuth2ConfigFromFields({
      oauthGrantType: 'client_credentials',
      oauthClientId: 'id',
      oauthClientSecret: 'secret',
      oauthAuthUrl: '',
      oauthTokenUrl: 'https://token',
      oauthScopes: 'scope1',
      oauthAccessToken: 'token',
      oauthRefreshToken: '',
      oauthTokenExpiry: null,
    })
    expect(config.grantType).toBe('client_credentials')
    expect(config.clientId).toBe('id')
    expect(config.clientSecret).toBe('secret')
    expect(config.tokenUrl).toBe('https://token')
    expect(config.scopes).toBe('scope1')
    expect(config.accessToken).toBe('token')
    expect(config.tokenExpiry).toBeNull()
  })
})
