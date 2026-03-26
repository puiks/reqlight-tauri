<script lang="ts">
  import HttpMethodBadge from "../shared/HttpMethodBadge.svelte";
  import type { SavedRequest } from "../../lib/types";

  let {
    request,
    isSelected = false,
    onclick,
    oncontextmenu,
  }: {
    request: SavedRequest;
    isSelected?: boolean;
    onclick: () => void;
    oncontextmenu?: (e: MouseEvent) => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="request-row"
  class:selected={isSelected}
  onclick={onclick}
  oncontextmenu={oncontextmenu}
>
  <HttpMethodBadge method={request.method} />
  <span class="name">{request.name}</span>
</div>

<style>
  .request-row {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
    padding: var(--sp-xs) var(--sp-sm) var(--sp-xs) var(--sp-xl);
    cursor: pointer;
    border-radius: var(--radius-sm);
    min-height: 28px;
  }
  .request-row:hover {
    background: var(--bg-hover);
  }
  .request-row.selected {
    background: var(--bg-selected);
  }
  .name {
    font-size: var(--fs-small);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
</style>
