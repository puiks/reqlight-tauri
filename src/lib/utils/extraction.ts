import type { ExtractionRule, ResponseRecord } from '../types'
import { extractByPath } from './jsonpath'
import { environmentStore } from '../stores/environment.svelte'

/**
 * Apply extraction rules to a JSON response, setting environment variables
 * for each matching rule.
 */
export function applyExtractionRules(
  rules: ExtractionRule[],
  response: ResponseRecord,
): void {
  if (!response.bodyString || !response.isJson) return
  const enabledRules = rules.filter((r) => r.isEnabled && r.variableName && r.jsonPath)
  if (enabledRules.length === 0) return

  let parsed: unknown
  try {
    parsed = JSON.parse(response.bodyString)
  } catch {
    return
  }

  for (const rule of enabledRules) {
    const value = extractByPath(parsed, rule.jsonPath)
    if (value !== undefined) {
      environmentStore.setVariable(rule.variableName, value)
    }
  }
}
