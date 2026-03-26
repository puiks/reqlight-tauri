<script lang="ts">
  import type { ResponseRecord } from "../../lib/types";
  import { formatJson, highlightJson } from "../../lib/utils/json-highlighter";

  let { response }: { response: ResponseRecord } = $props();

  let showFormatted = $state(true);

  const bodyText = $derived(response.bodyString ?? "(No body)");
  const formattedJson = $derived(
    response.isJson ? formatJson(bodyText) : bodyText,
  );
  const highlightedHtml = $derived(
    response.isJson && showFormatted
      ? highlightJson(formattedJson)
      : null,
  );

  function handleCopy() {
    const text = showFormatted && response.isJson ? formattedJson : bodyText;
    navigator.clipboard.writeText(text);
  }
</script>

<div class="response-body">
  <div class="toolbar">
    {#if response.isJson}
      <button
        class="toggle"
        class:active={showFormatted}
        onclick={() => (showFormatted = !showFormatted)}
      >
        {showFormatted ? "Formatted" : "Raw"}
      </button>
    {/if}
    <button class="copy-btn" onclick={handleCopy} title="Copy response body">
      Copy
    </button>
  </div>

  {#if response.isTruncated}
    <div class="truncation-warning">
      Response truncated — body is {(response.bodySize / 1024 / 1024).toFixed(1)} MB, only first 5 MB shown.
    </div>
  {/if}

  <div class="body-content">
    {#if highlightedHtml}
      <pre class="highlighted"><code>{@html highlightedHtml}</code></pre>
    {:else}
      <pre class="plain"><code>{bodyText}</code></pre>
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
    background: rgba(245, 158, 11, 0.1);
    color: var(--color-warning);
    font-size: var(--fs-caption);
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
  }
</style>
