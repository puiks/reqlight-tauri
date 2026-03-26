<script lang="ts">
  let {
    title,
    message,
    confirmLabel = "Delete",
    onconfirm,
    oncancel,
  }: {
    title: string;
    message: string;
    confirmLabel?: string;
    onconfirm: () => void;
    oncancel: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") oncancel();
    if (e.key === "Enter") onconfirm();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={oncancel}>
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <h3>{title}</h3>
    <p>{message}</p>
    <div class="actions">
      <button class="cancel" onclick={oncancel}>Cancel</button>
      <button class="confirm" onclick={onconfirm}>{confirmLabel}</button>
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
    z-index: 200;
  }
  .dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--sp-xl);
    min-width: 320px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }
  h3 {
    font-size: var(--fs-callout);
    font-weight: 600;
    margin-bottom: var(--sp-sm);
  }
  p {
    font-size: var(--fs-body);
    color: var(--text-secondary);
    margin-bottom: var(--sp-lg);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-sm);
  }
  .cancel {
    background: var(--bg-tertiary);
    padding: var(--sp-xs) var(--sp-md);
  }
  .confirm {
    background: var(--color-error);
    color: white;
    padding: var(--sp-xs) var(--sp-md);
  }
  .confirm:hover {
    opacity: 0.9;
    background: var(--color-error);
  }
</style>
