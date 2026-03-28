/**
 * Simple line-based diff using the LCS (Longest Common Subsequence) algorithm.
 * Returns an array of diff lines with their type.
 */

export type DiffLineType = "same" | "added" | "removed";

export interface DiffLine {
  type: DiffLineType;
  content: string;
  lineNumber: number | null; // line number in the respective source
}

export interface DiffResult {
  left: DiffLine[];
  right: DiffLine[];
  hasChanges: boolean;
}

/**
 * Compute a line-by-line diff between two strings.
 * Returns parallel arrays for left (old) and right (new) with alignment.
 */
export function computeDiff(oldText: string, newText: string): DiffResult {
  const oldLines = oldText.split("\n");
  const newLines = newText.split("\n");

  const lcs = computeLCS(oldLines, newLines);

  const left: DiffLine[] = [];
  const right: DiffLine[] = [];
  let hasChanges = false;

  let oi = 0;
  let ni = 0;
  let li = 0;

  while (oi < oldLines.length || ni < newLines.length) {
    if (li < lcs.length && oi < oldLines.length && ni < newLines.length && oldLines[oi] === lcs[li] && newLines[ni] === lcs[li]) {
      // Common line
      left.push({ type: "same", content: oldLines[oi], lineNumber: oi + 1 });
      right.push({ type: "same", content: newLines[ni], lineNumber: ni + 1 });
      oi++;
      ni++;
      li++;
    } else {
      // Consume removed lines from old
      if (oi < oldLines.length && (li >= lcs.length || oldLines[oi] !== lcs[li])) {
        left.push({ type: "removed", content: oldLines[oi], lineNumber: oi + 1 });
        right.push({ type: "removed", content: "", lineNumber: null });
        oi++;
        hasChanges = true;
      }
      // Consume added lines from new
      else if (ni < newLines.length && (li >= lcs.length || newLines[ni] !== lcs[li])) {
        left.push({ type: "added", content: "", lineNumber: null });
        right.push({ type: "added", content: newLines[ni], lineNumber: ni + 1 });
        ni++;
        hasChanges = true;
      }
    }
  }

  return { left, right, hasChanges };
}

/**
 * Compute the Longest Common Subsequence of two string arrays.
 * Uses standard DP approach. For very large inputs this could be slow,
 * but response bodies are capped at 5MB / ~100K lines which is acceptable.
 */
function computeLCS(a: string[], b: string[]): string[] {
  const m = a.length;
  const n = b.length;

  // For large inputs, fall back to a simpler approach
  if (m * n > 1_000_000) {
    return simpleLCS(a, b);
  }

  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (a[i - 1] === b[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to get the LCS
  const result: string[] = [];
  let i = m;
  let j = n;
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) {
      result.unshift(a[i - 1]);
      i--;
      j--;
    } else if (dp[i - 1][j] > dp[i][j - 1]) {
      i--;
    } else {
      j--;
    }
  }

  return result;
}

/**
 * Simple LCS for large inputs: just find matching lines greedily.
 * Not optimal but avoids O(m*n) memory.
 */
function simpleLCS(a: string[], b: string[]): string[] {
  const bIndex = new Map<string, number[]>();
  for (let j = 0; j < b.length; j++) {
    const existing = bIndex.get(b[j]);
    if (existing) {
      existing.push(j);
    } else {
      bIndex.set(b[j], [j]);
    }
  }

  const result: string[] = [];
  let lastJ = -1;
  for (let i = 0; i < a.length; i++) {
    const positions = bIndex.get(a[i]);
    if (positions) {
      const nextJ = positions.find((j) => j > lastJ);
      if (nextJ !== undefined) {
        result.push(a[i]);
        lastJ = nextJ;
      }
    }
  }
  return result;
}
