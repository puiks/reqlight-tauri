<script lang="ts">
  import type { ResponseRecord } from "../../lib/types";
  import { toastStore } from "../../lib/stores/toast.svelte";

  let { response }: { response: ResponseRecord } = $props();

  function copyAll() {
    const text = response.headers.map((h) => `${h.key}: ${h.value}`).join("\n");
    navigator.clipboard.writeText(text);
    toastStore.show("Headers copied");
  }

  function copyValue(value: string) {
    navigator.clipboard.writeText(value);
    toastStore.show("Copied");
  }
</script>

<div class="response-headers">
  <div class="header-toolbar">
    <button class="copy-all-btn" onclick={copyAll} title="Copy all headers">Copy All</button>
  </div>
  <table>
    <thead>
      <tr>
        <th>Name</th>
        <th>Value</th>
        <th class="action-col"></th>
      </tr>
    </thead>
    <tbody>
      {#each response.headers as header}
        <tr>
          <td class="header-name">{header.key}</td>
          <td class="header-value">{header.value}</td>
          <td class="action-col">
            <button
              class="copy-value-btn"
              onclick={() => copyValue(header.value)}
              title="Copy value"
            >
              ⎘
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .response-headers {
    padding: var(--sp-sm);
    overflow: auto;
    height: 100%;
  }
  .header-toolbar {
    display: flex;
    justify-content: flex-end;
    padding-bottom: var(--sp-xs);
  }
  .copy-all-btn {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    padding: 2px var(--sp-sm);
    border-radius: var(--radius-sm);
  }
  .copy-all-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--fs-small);
  }
  th {
    text-align: left;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-sm);
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: var(--fs-footnote);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  td {
    padding: var(--sp-xs) var(--sp-sm);
    border-bottom: 1px solid var(--border-light);
    user-select: text;
  }
  .header-name {
    font-weight: 500;
    color: var(--text-primary);
    width: 200px;
    font-family: var(--font-mono);
  }
  .header-value {
    color: var(--text-secondary);
    font-family: var(--font-mono);
    word-break: break-all;
  }
  .action-col {
    width: 28px;
    text-align: center;
    padding: 0;
  }
  .copy-value-btn {
    font-size: var(--fs-callout);
    color: var(--text-tertiary);
    opacity: 0;
    transition: opacity var(--transition-fast);
    padding: 2px;
  }
  tr:hover .copy-value-btn {
    opacity: 1;
  }
  .copy-value-btn:hover {
    color: var(--text-primary);
  }
</style>
