<script lang="ts">
  import type { ResponseRecord } from "../../lib/types";

  let { response }: { response: ResponseRecord } = $props();

  const isSuccess = $derived(
    response.statusCode >= 200 && response.statusCode < 300,
  );
  const isError = $derived(response.statusCode >= 400);

  function formatTime(ms: number): string {
    return ms < 1000 ? `${Math.round(ms)} ms` : `${(ms / 1000).toFixed(2)} s`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="status-bar">
  <span
    class="status-code"
    class:success={isSuccess}
    class:error={isError}
    class:warning={!isSuccess && !isError}
  >
    {response.statusCode}
  </span>
  <span class="divider">|</span>
  <span class="stat">{formatTime(response.elapsedTime)}</span>
  <span class="divider">|</span>
  <span class="stat">{formatSize(response.bodySize)}</span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
    padding: var(--sp-sm) var(--sp-md);
    border-bottom: 1px solid var(--border-color);
    font-size: var(--fs-small);
    font-family: var(--font-mono);
  }
  .status-code {
    font-weight: 700;
    padding: 1px var(--sp-sm);
    border-radius: var(--radius-sm);
  }
  .status-code.success {
    color: var(--color-success);
    background: var(--color-success-overlay);
  }
  .status-code.error {
    color: var(--color-error);
    background: var(--color-error-overlay);
  }
  .status-code.warning {
    color: var(--color-warning);
    background: var(--color-warning-overlay);
  }
  .divider {
    color: var(--text-tertiary);
  }
  .stat {
    color: var(--text-secondary);
  }
</style>
