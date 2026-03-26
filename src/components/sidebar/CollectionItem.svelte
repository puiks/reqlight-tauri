<script lang="ts">
  import RequestRow from "./RequestRow.svelte";
  import type { RequestCollection } from "../../lib/types";
  import { appStore } from "../../lib/stores/app.svelte";
  import { editorStore } from "../../lib/stores/editor.svelte";

  let {
    collection,
    oncontextmenu,
  }: {
    collection: RequestCollection;
    oncontextmenu?: (e: MouseEvent) => void;
  } = $props();

  let expanded = $state(true);

  function toggleExpand() {
    expanded = !expanded;
  }

  function selectRequest(requestId: string) {
    appStore.selectRequest(collection.id, requestId);
    const request = collection.requests.find((r) => r.id === requestId);
    if (request) editorStore.loadFrom(request);
  }
</script>

<div class="collection">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="header" onclick={toggleExpand} oncontextmenu={oncontextmenu}>
    <span class="chevron" class:expanded>{expanded ? "▾" : "▸"}</span>
    <span class="name">{collection.name}</span>
    <span class="count">{collection.requests.length}</span>
  </div>
  {#if expanded}
    <div class="requests">
      {#each collection.requests as request (request.id)}
        <RequestRow
          {request}
          isSelected={appStore.selectedRequestId === request.id}
          onclick={() => selectRequest(request.id)}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .collection {
    margin-bottom: 2px;
  }
  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: var(--sp-xs) var(--sp-sm);
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: var(--fs-small);
    font-weight: 600;
  }
  .header:hover {
    background: var(--bg-hover);
  }
  .chevron {
    font-size: 10px;
    color: var(--text-secondary);
    width: 12px;
    text-align: center;
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    font-weight: 400;
  }
  .requests {
    padding-left: var(--sp-xs);
  }
</style>
