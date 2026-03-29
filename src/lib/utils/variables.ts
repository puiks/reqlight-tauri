import type { KeyValuePair, RequestBody, AuthConfig, SavedRequest } from '../types'

const VARIABLE_PATTERN = /\{\{([^}]+)\}\}/g

/** Extract all `{{variable}}` names from a string. */
export function extractVariableNames(input: string): string[] {
  const names: string[] = []
  let match: RegExpExecArray | null
  while ((match = VARIABLE_PATTERN.exec(input)) !== null) {
    const name = match[1].trim()
    if (name && !names.includes(name)) {
      names.push(name)
    }
  }
  VARIABLE_PATTERN.lastIndex = 0
  return names
}

/** Collect all variable names referenced in a request's fields. */
export function collectRequestVariables(request: SavedRequest): string[] {
  const parts: string[] = [request.url]

  for (const h of request.headers ?? []) {
    if (h.isEnabled) {
      parts.push(h.key, h.value)
    }
  }
  for (const p of request.queryParams ?? []) {
    if (p.isEnabled) {
      parts.push(p.key, p.value)
    }
  }

  collectBodyVariables(request.body, parts)
  collectAuthVariables(request.auth, parts)

  const combined = parts.join('\n')
  return extractVariableNames(combined)
}

function collectBodyVariables(body: RequestBody, parts: string[]) {
  if (body === 'none') return
  if ('json' in body) parts.push(body.json)
  else if ('rawText' in body) parts.push(body.rawText)
  else if ('graphql' in body) {
    parts.push(body.graphql.query, body.graphql.variables)
  } else if ('formData' in body) {
    for (const p of body.formData) {
      if (p.isEnabled) parts.push(p.key, p.value)
    }
  } else if ('multipart' in body) {
    for (const p of body.multipart) {
      if (p.isEnabled) parts.push(p.name, p.value)
    }
  }
}

function collectAuthVariables(auth: AuthConfig | undefined, parts: string[]) {
  if (!auth || auth === 'none') return
  if ('bearerToken' in auth) parts.push(auth.bearerToken.token)
  else if ('basicAuth' in auth) {
    parts.push(auth.basicAuth.username, auth.basicAuth.password)
  } else if ('apiKey' in auth) parts.push(auth.apiKey.value)
  else if ('oauth2' in auth) {
    const o = auth.oauth2
    if (o.accessToken) parts.push(o.accessToken)
  }
}

/** Find variable names used in a request that don't exist in the environment. */
export function findUnmatchedVariables(
  request: SavedRequest,
  envVariables: KeyValuePair[],
): string[] {
  const used = collectRequestVariables(request)
  if (used.length === 0) return []

  const available = new Set(envVariables.filter((v) => v.isEnabled && v.key).map((v) => v.key))
  return used.filter((name) => !available.has(name))
}
