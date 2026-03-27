export interface SearchMatch {
  start: number;
  end: number;
}

/**
 * Find all occurrences of a query in text.
 * Returns an array of { start, end } positions.
 */
export function findMatches(
  text: string,
  query: string,
  caseSensitive = false,
): SearchMatch[] {
  if (!query) return [];
  const matches: SearchMatch[] = [];
  const haystack = caseSensitive ? text : text.toLowerCase();
  const needle = caseSensitive ? query : query.toLowerCase();
  let idx = 0;
  while (idx < haystack.length) {
    const found = haystack.indexOf(needle, idx);
    if (found === -1) break;
    matches.push({ start: found, end: found + needle.length });
    idx = found + 1;
  }
  return matches;
}

/**
 * Escape HTML special characters to prevent XSS when inserting into innerHTML.
 */
function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Highlight matches in plain text by wrapping them with <mark> tags.
 * Returns HTML string safe for {@html}.
 */
export function highlightMatches(
  text: string,
  matches: SearchMatch[],
  activeIndex = -1,
): string {
  if (matches.length === 0) return escapeHtml(text);
  const parts: string[] = [];
  let lastEnd = 0;
  for (let i = 0; i < matches.length; i++) {
    const m = matches[i];
    if (m.start > lastEnd) {
      parts.push(escapeHtml(text.slice(lastEnd, m.start)));
    }
    const cls = i === activeIndex ? "search-match active" : "search-match";
    parts.push(`<mark class="${cls}">${escapeHtml(text.slice(m.start, m.end))}</mark>`);
    lastEnd = m.end;
  }
  if (lastEnd < text.length) {
    parts.push(escapeHtml(text.slice(lastEnd)));
  }
  return parts.join("");
}
