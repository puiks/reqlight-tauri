<script lang="ts">
  let {
    x,
    y,
    type,
    id,
    onaddrequest,
    onrename,
    ondelete,
  }: {
    x: number;
    y: number;
    type: "collection" | "request";
    id: string;
    onaddrequest: (id: string) => void;
    onrename: (id: string) => void;
    ondelete: (id: string) => void;
  } = $props();
</script>

<div
  class="context-menu"
  style="left: {x}px; top: {y}px"
  role="menu"
>
  {#if type === "collection"}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="menu-item" role="menuitem" tabindex="-1" onclick={() => onaddrequest(id)}>
      New Request
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="menu-item" role="menuitem" tabindex="-1" onclick={() => onrename(id)}>
      Rename
    </div>
    <div class="menu-divider"></div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="menu-item danger" role="menuitem" tabindex="-1" onclick={() => ondelete(id)}>
      Delete
    </div>
  {/if}
</div>

<style>
  .context-menu {
    position: fixed;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    padding: var(--sp-xs) 0;
    min-width: 160px;
    z-index: 50;
  }
  .menu-item {
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-small);
    cursor: pointer;
  }
  .menu-item:hover {
    background: var(--bg-hover);
  }
  .menu-item.danger {
    color: var(--color-error);
  }
  .menu-divider {
    height: 1px;
    background: var(--border-color);
    margin: var(--sp-xs) 0;
  }
</style>
