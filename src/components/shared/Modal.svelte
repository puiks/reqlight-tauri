<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    onclose,
    children,
    wide = false,
  }: { title: string; onclose: () => void; children: Snippet; wide?: boolean } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onclose}>
  <div class="modal" class:wide onclick={(e) => e.stopPropagation()}>
    <div class="header">
      <h2>{title}</h2>
      <button class="close-btn" onclick={onclose} aria-label="Close">✕</button>
    </div>
    <div class="body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-backdrop);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-modal);
    min-width: 500px;
    max-width: 90vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-md) var(--sp-lg);
    border-bottom: 1px solid var(--border-color);
  }
  .header h2 {
    font-size: var(--fs-callout);
    font-weight: 600;
  }
  .close-btn {
    font-size: var(--fs-callout);
    color: var(--text-secondary);
    padding: var(--sp-xs);
  }
  .body {
    padding: var(--sp-lg);
    overflow-y: auto;
    flex: 1;
  }
  .modal.wide {
    min-width: 800px;
    width: 80vw;
  }
</style>
