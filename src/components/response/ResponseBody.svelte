<script lang="ts">
  import { HIGHLIGHT_SIZE_LIMIT } from "../../lib/constants";
  import type { ResponseRecord } from "../../lib/types";
  import { formatJson, highlightJson } from "../../lib/utils/json-highlighter";
  import { formatXml, highlightXml } from "../../lib/utils/xml-highlighter";
  import { findMatches, highlightMatches } from "../../lib/utils/text-search";
  import ResponseSearch from "./ResponseSearch.svelte";
  import { toastStore } from "../../lib/stores/toast.svelte";

  let { response }: { response: ResponseRecord } = $props();

  type ContentCategory = "json" | "xml" | "html" | "image" | "text";

  let showFormatted = $state(true);
  let searchQuery = $state("");
  let searchVisible = $state(false);
  let activeMatchIndex = $state(0);
  let searchInputEl = $state<HTMLInputElement | null>(null);
  let htmlPreviewMode = $state<"preview" | "source">("preview");

  const contentCategory = $derived((): ContentCategory => {
    const ct = (response.contentType ?? "").toLowerCase();
    if (response.isJson || ct.includes("json")) return "json";
    if (ct.includes("xml")) return "xml";
    if (ct.includes("html")) return "html";
    if (ct.startsWith("image/")) return "image";
    return "text";
  });

  const bodyText = $derived(response.bodyString ?? "(No body)");

  const displayText = $derived((): string => {
    const cat = contentCategory();
    if (!showFormatted) return bodyText;
    if (cat === "json") return formatJson(bodyText);
    if (cat === "xml") return formatXml(bodyText);
    return bodyText;
  });

  const isBodyTooLarge = $derived(bodyText.length > HIGHLIGHT_SIZE_LIMIT);

  const syntaxHtml = $derived((): string | null => {
    if (!showFormatted || searchQuery || isBodyTooLarge) return null;
    const cat = contentCategory();
    if (cat === "json") return highlightJson(displayText());
    if (cat === "xml") return highlightXml(displayText());
    if (cat === "html" && htmlPreviewMode === "source") return highlightXml(bodyText);
    return null;
  });

  const searchResults = $derived(findMatches(displayText(), searchQuery));

  $effect(() => {
    if (searchResults.length > 0 && activeMatchIndex >= searchResults.length) {
      activeMatchIndex = 0;
    }
  });

  const searchHighlightedHtml = $derived(
    searchQuery && searchResults.length > 0
      ? highlightMatches(displayText(), searchResults, activeMatchIndex)
      : null,
  );

  const imageDataUrl = $derived((): string | null => {
    if (contentCategory() !== "image" || !response.bodyString) return null;
    const ct = response.contentType || "image/png";
    if (ct.includes("svg")) {
      return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(response.bodyString)}`;
    }
    return null;
  });

  const isFormattable = $derived((): boolean => {
    const cat = contentCategory();
    return cat === "json" || cat === "xml";
  });

  function handleCopy() {
    navigator.clipboard.writeText(displayText());
    toastStore.show("Body copied");
  }

  function toggleSearch() {
    searchVisible = !searchVisible;
    if (!searchVisible) {
      searchQuery = "";
      activeMatchIndex = 0;
    } else {
      requestAnimationFrame(() => searchInputEl?.focus());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      toggleSearch();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="response-body">
  <div class="toolbar">
    {#if isFormattable()}
      <button
        class="toggle"
        class:active={showFormatted}
        onclick={() => (showFormatted = !showFormatted)}
      >
        {showFormatted ? "Formatted" : "Raw"}
      </button>
    {/if}
    {#if contentCategory() === "html"}
      <button
        class="toggle"
        class:active={htmlPreviewMode === "preview"}
        onclick={() => (htmlPreviewMode = htmlPreviewMode === "preview" ? "source" : "preview")}
      >
        {htmlPreviewMode === "preview" ? "Preview" : "Source"}
      </button>
    {/if}
    <span class="content-type-badge">{contentCategory()}</span>
    <button class="search-btn" onclick={toggleSearch} title="Search (⌘F)" aria-label="Search">
      ⌕
    </button>
    <button class="copy-btn" onclick={handleCopy} title="Copy response body">
      Copy
    </button>
  </div>

  {#if searchVisible}
    <ResponseSearch
      bind:searchQuery
      {searchResults}
      bind:activeMatchIndex
      bind:searchInputEl
      onclose={toggleSearch}
    />
  {/if}

  {#if response.isTruncated}
    <div class="truncation-warning">
      Response truncated — body is {(response.bodySize / 1024 / 1024).toFixed(1)} MB, only first 5 MB shown.
    </div>
  {/if}

  {#if isBodyTooLarge && !response.isTruncated}
    <div class="truncation-warning">
      Syntax highlighting disabled — body exceeds 512 KB.
    </div>
  {/if}

  <div class="body-content">
    {#if contentCategory() === "image"}
      {#if imageDataUrl()}
        <div class="image-preview">
          <img src={imageDataUrl()} alt="Response" />
        </div>
      {:else}
        <div class="image-fallback">
          <span class="text-secondary">Binary image ({response.contentType}) — {(response.bodySize / 1024).toFixed(1)} KB</span>
        </div>
      {/if}
    {:else if contentCategory() === "html" && htmlPreviewMode === "preview"}
      <iframe
        class="html-preview"
        srcdoc={bodyText}
        sandbox=""
        title="HTML Preview"
      ></iframe>
    {:else if searchHighlightedHtml}
      <pre class="highlighted"><code>{@html searchHighlightedHtml}</code></pre>
    {:else if syntaxHtml()}
      <pre class="highlighted"><code>{@html syntaxHtml()}</code></pre>
    {:else}
      <pre class="plain"><code>{displayText()}</code></pre>
    {/if}
  </div>
</div>

<style>
  .response-body {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
    padding: var(--sp-xs) var(--sp-md);
    border-bottom: 1px solid var(--border-light);
  }
  .toggle {
    font-size: var(--fs-caption);
    padding: 2px var(--sp-sm);
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
  }
  .toggle.active {
    background: var(--bg-selected);
    color: var(--color-info);
  }
  .content-type-badge {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-weight: 600;
  }
  .search-btn {
    font-size: var(--fs-callout);
    color: var(--text-secondary);
    padding: 0 var(--sp-xs);
  }
  .copy-btn {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    margin-left: auto;
  }
  .body-content {
    flex: 1;
    overflow: auto;
    padding: var(--sp-sm) var(--sp-md);
  }
  pre {
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    user-select: text;
  }
  .truncation-warning {
    padding: var(--sp-xs) var(--sp-md);
    background: var(--color-warning-overlay);
    color: var(--color-warning);
    font-size: var(--fs-caption);
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
  }
  .html-preview {
    width: 100%;
    height: 100%;
    border: none;
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
  }
  .image-preview {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: var(--sp-md);
  }
  .image-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
  .image-fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: var(--fs-small);
  }
  .text-secondary {
    color: var(--text-secondary);
  }
  :global(.search-match) {
    background: var(--color-warning-highlight);
    border-radius: 1px;
  }
  :global(.search-match.active) {
    background: var(--color-warning-highlight-active);
    outline: 1px solid var(--color-warning);
  }
</style>
