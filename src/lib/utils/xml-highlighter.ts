/**
 * Simple XML/HTML syntax highlighter.
 * Returns HTML with span classes for styling.
 */
import { escapeHtml } from "./html";


export function highlightXml(xml: string): string {
  // Regex-based tokenizer for XML/HTML
  return xml.replace(
    /(<\/?)(\w[\w.-]*)([^>]*?)(\/?>)|<!--[\s\S]*?-->|([^<]+)/g,
    (match, open, tag, attrs, close, text) => {
      if (match.startsWith("<!--")) {
        return `<span class="xml-comment">${escapeHtml(match)}</span>`;
      }
      if (text !== undefined) {
        return escapeHtml(text);
      }
      // Highlight tag name and attributes
      const highlightedAttrs = attrs.replace(
        /(\s+)([\w:.-]+)(=)("[^"]*"|'[^']*')/g,
        (_: string, ws: string, name: string, eq: string, val: string) =>
          `${ws}<span class="xml-attr">${escapeHtml(name)}</span>${eq}<span class="xml-value">${escapeHtml(val.slice(1, -1))}</span>`,
      );
      return `<span class="xml-bracket">${escapeHtml(open)}</span><span class="xml-tag">${escapeHtml(tag)}</span>${highlightedAttrs}<span class="xml-bracket">${escapeHtml(close)}</span>`;
    },
  );
}

/**
 * Try to format XML with indentation.
 * Best-effort — doesn't handle all edge cases.
 */
export function formatXml(xml: string): string {
  const INDENT = "  ";
  let formatted = "";
  let indent = 0;
  // Split by tags
  const tokens = xml.replace(/>\s*</g, ">\n<").split("\n");

  for (const token of tokens) {
    const trimmed = token.trim();
    if (!trimmed) continue;

    if (trimmed.startsWith("</")) {
      indent = Math.max(0, indent - 1);
      formatted += INDENT.repeat(indent) + trimmed + "\n";
    } else if (trimmed.startsWith("<?") || trimmed.startsWith("<!")) {
      formatted += INDENT.repeat(indent) + trimmed + "\n";
    } else if (trimmed.endsWith("/>")) {
      formatted += INDENT.repeat(indent) + trimmed + "\n";
    } else if (trimmed.startsWith("<") && !trimmed.startsWith("</")) {
      formatted += INDENT.repeat(indent) + trimmed + "\n";
      // Only increase indent if this isn't a self-closing or void tag
      if (!trimmed.includes("</")) {
        indent++;
      }
    } else {
      formatted += INDENT.repeat(indent) + trimmed + "\n";
    }
  }
  return formatted.trimEnd();
}
