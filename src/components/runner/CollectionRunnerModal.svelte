<script lang="ts">
  import Modal from "../shared/Modal.svelte";
  import { runnerStore } from "../../lib/stores/runner.svelte";
  import { METHOD_COLORS } from "../../lib/types";

  let { onclose }: { onclose: () => void } = $props();

  function handleClose() {
    if (runnerStore.status !== "running") {
      runnerStore.reset();
    }
    onclose();
  }

  function formatTime(ms: number | null): string {
    if (ms === null) return "—";
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }
</script>

<Modal title="Collection Runner — {runnerStore.collectionName}" onclose={handleClose}>
  <div class="runner">
    <!-- Progress -->
    {#if runnerStore.status === "running"}
      <div class="progress-section">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {runnerStore.progress * 100}%"></div>
        </div>
        <div class="progress-label">
          Running {runnerStore.currentIndex + 1} / {runnerStore.totalRequests}
        </div>
      </div>
    {/if}

    <!-- Summary -->
    {#if runnerStore.status === "completed" || runnerStore.status === "stopped"}
      <div class="summary">
        <span class="summary-item pass">{runnerStore.passCount} passed</span>
        <span class="summary-item fail">{runnerStore.failCount} failed</span>
        <span class="summary-item time">{formatTime(runnerStore.totalElapsed)}</span>
        {#if runnerStore.status === "stopped"}
          <span class="summary-item stopped">Stopped</span>
        {/if}
      </div>
    {/if}

    <!-- Results -->
    {#if runnerStore.results.length > 0}
      <div class="results">
        {#each runnerStore.results as result (result.requestId)}
          <div class="result-row" class:failed={!result.passed}>
            <span class="result-status">
              {#if result.passed}
                <span class="icon pass">✓</span>
              {:else}
                <span class="icon fail">✗</span>
              {/if}
            </span>
            <span class="result-method" style="color: {METHOD_COLORS[result.method]}">
              {result.method}
            </span>
            <span class="result-name">{result.requestName}</span>
            <span class="result-code">
              {#if result.statusCode !== null}
                {result.statusCode}
              {:else}
                ERR
              {/if}
            </span>
            <span class="result-time">{formatTime(result.elapsedTime)}</span>
          </div>
          {#if result.errorMessage}
            <div class="result-error">{result.errorMessage}</div>
          {/if}
        {/each}
      </div>
    {/if}

    <!-- Actions -->
    <div class="actions">
      {#if runnerStore.status === "running"}
        <button class="btn stop" onclick={() => runnerStore.stop()}>Stop</button>
      {:else}
        <button class="btn close" onclick={handleClose}>Close</button>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .runner {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
    min-width: 500px;
  }
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
  }
  .progress-bar {
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: var(--color-info);
    border-radius: 3px;
    transition: width 0.2s ease;
  }
  .progress-label {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    text-align: center;
  }
  .summary {
    display: flex;
    gap: var(--sp-md);
    justify-content: center;
    padding: var(--sp-sm) 0;
  }
  .summary-item {
    font-size: var(--fs-small);
    font-weight: 600;
  }
  .summary-item.pass {
    color: var(--color-success);
  }
  .summary-item.fail {
    color: var(--color-error);
  }
  .summary-item.time {
    color: var(--text-secondary);
  }
  .summary-item.stopped {
    color: var(--color-warning);
  }
  .results {
    display: flex;
    flex-direction: column;
    max-height: 400px;
    overflow-y: auto;
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
  }
  .result-row {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
    padding: var(--sp-xs) var(--sp-sm);
    font-size: var(--fs-small);
    border-bottom: 1px solid var(--border-light);
  }
  .result-row:last-child {
    border-bottom: none;
  }
  .result-row.failed {
    background: color-mix(in srgb, var(--color-error) 5%, transparent);
  }
  .result-status {
    width: 20px;
    text-align: center;
    flex-shrink: 0;
  }
  .icon.pass {
    color: var(--color-success);
    font-weight: 700;
  }
  .icon.fail {
    color: var(--color-error);
    font-weight: 700;
  }
  .result-method {
    font-size: var(--fs-caption);
    font-weight: 700;
    width: 50px;
    flex-shrink: 0;
  }
  .result-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .result-code {
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-secondary);
    width: 36px;
    text-align: right;
    flex-shrink: 0;
  }
  .result-time {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    width: 60px;
    text-align: right;
    flex-shrink: 0;
  }
  .result-error {
    padding: 2px var(--sp-sm) var(--sp-xs) calc(20px + var(--sp-sm) + var(--sp-sm));
    font-size: var(--fs-caption);
    color: var(--color-error);
    word-break: break-word;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-sm);
  }
  .btn {
    padding: var(--sp-xs) var(--sp-lg);
    font-size: var(--fs-small);
    font-weight: 600;
    border-radius: var(--radius-sm);
  }
  .btn.stop {
    background: var(--color-error);
    color: white;
  }
  .btn.close {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
</style>
