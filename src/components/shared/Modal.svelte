<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    onclose,
    children,
  }: { title: string; onclose: () => void; children: Snippet } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onclose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="header">
      <h2>{title}</h2>
      <button class="close-btn" onclick={onclose}>✕</button>
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
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
    min-width: 500px;
    max-width: 90vw;
    max-height: 80vh;
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
</style>
