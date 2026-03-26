<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import HttpMethodBadge from "../shared/HttpMethodBadge.svelte";

  let expanded = $state(false);

  const recentHistory = $derived(appStore.history.slice(0, 10));
</script>

{#if appStore.history.length > 0}
  <div class="history-section">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="header" onclick={() => (expanded = !expanded)}>
      <span class="chevron">{expanded ? "▾" : "▸"}</span>
      <span class="title">History</span>
      <span class="count">{appStore.history.length}</span>
    </div>
    {#if expanded}
      <div class="entries">
        {#each recentHistory as entry (entry.id)}
          <div class="entry">
            <HttpMethodBadge method={entry.method} />
            <span class="url">{entry.url}</span>
            {#if entry.statusCode}
              <span
                class="status"
                class:success={entry.statusCode >= 200 &&
                  entry.statusCode < 300}
                class:error={entry.statusCode >= 400}
              >
                {entry.statusCode}
              </span>
            {/if}
          </div>
        {/each}
        <button class="clear-btn" onclick={() => appStore.clearHistory()}>
          Clear History
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .history-section {
    border-top: 1px solid var(--border-color);
    margin-top: var(--sp-sm);
    padding-top: var(--sp-xs);
  }
  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: var(--sp-xs) var(--sp-sm);
    cursor: pointer;
    font-size: var(--fs-small);
    font-weight: 600;
    color: var(--text-secondary);
  }
  .header:hover {
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
  }
  .chevron {
    font-size: 10px;
    width: 12px;
    text-align: center;
  }
  .title {
    flex: 1;
  }
  .count {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    font-weight: 400;
  }
  .entries {
    padding: 0 var(--sp-sm);
  }
  .entry {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: 3px var(--sp-sm);
    font-size: var(--fs-footnote);
    border-radius: var(--radius-sm);
  }
  .url {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }
  .status {
    font-family: var(--font-mono);
    font-size: var(--fs-caption);
  }
  .status.success {
    color: var(--color-success);
  }
  .status.error {
    color: var(--color-error);
  }
  .clear-btn {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    margin-top: var(--sp-xs);
    padding: 2px var(--sp-sm);
  }
</style>
