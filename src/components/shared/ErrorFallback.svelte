<script lang="ts">
  interface Props {
    error: unknown;
    onreset: () => void;
  }

  const { error, onreset }: Props = $props();

  const message = $derived(
    error instanceof Error ? error.message : String(error),
  );
</script>

<div class="error-fallback">
  <div class="error-icon">!</div>
  <h2>Something went wrong</h2>
  <p class="error-message">{message}</p>
  <div class="actions">
    <button class="btn-retry" onclick={onreset}>Retry</button>
    <button class="btn-reload" onclick={() => window.location.reload()}>
      Reload App
    </button>
  </div>
</div>

<style>
  .error-fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: var(--sp-xl);
    text-align: center;
    color: var(--color-text-secondary);
  }

  .error-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-error);
    color: white;
    font-size: 24px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--sp-lg);
  }

  h2 {
    margin: 0 0 var(--sp-sm);
    color: var(--color-text-primary);
    font-size: var(--font-size-lg);
  }

  .error-message {
    margin: 0 0 var(--sp-lg);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    max-width: 500px;
    word-break: break-word;
  }

  .actions {
    display: flex;
    gap: var(--sp-sm);
  }

  .actions button {
    padding: var(--sp-sm) var(--sp-lg);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .btn-retry {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .btn-retry:hover {
    background: var(--color-bg-tertiary);
  }

  .btn-reload {
    background: var(--color-info);
    color: white;
    border-color: var(--color-info);
  }

  .btn-reload:hover {
    opacity: 0.9;
  }
</style>
