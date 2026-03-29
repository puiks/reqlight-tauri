<script lang="ts">
  import Modal from "../shared/Modal.svelte";
  import { computeDiff, type DiffResult } from "../../lib/utils/diff";
  import type { ResponseRecord } from "../../lib/types";

  let {
    pinnedResponse,
    currentResponse,
    onclose,
  }: {
    pinnedResponse: ResponseRecord;
    currentResponse: ResponseRecord;
    onclose: () => void;
  } = $props();

  const diff: DiffResult = $derived(
    computeDiff(
      formatBody(pinnedResponse),
      formatBody(currentResponse),
    )
  );

  const statusChanged = $derived(pinnedResponse.statusCode !== currentResponse.statusCode);
  const sizeChanged = $derived(pinnedResponse.bodySize !== currentResponse.bodySize);
  const timeChanged = $derived(pinnedResponse.elapsedTime !== currentResponse.elapsedTime);

  function formatBody(resp: ResponseRecord): string {
    if (!resp.bodyString) return "";
    if (resp.isJson) {
      try {
        return JSON.stringify(JSON.parse(resp.bodyString), null, 2);
      } catch {
        return resp.bodyString;
      }
    }
    return resp.bodyString;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<Modal title="Compare Responses" {onclose} wide>
  <div class="diff-modal">
    <div class="summary">
      <div class="summary-item" class:changed={statusChanged}>
        <span class="summary-label">Status</span>
        <span class="old-val">{pinnedResponse.statusCode}</span>
        <span class="arrow">→</span>
        <span class="new-val">{currentResponse.statusCode}</span>
      </div>
      <div class="summary-item" class:changed={timeChanged}>
        <span class="summary-label">Time</span>
        <span class="old-val">{pinnedResponse.elapsedTime}ms</span>
        <span class="arrow">→</span>
        <span class="new-val">{currentResponse.elapsedTime}ms</span>
      </div>
      <div class="summary-item" class:changed={sizeChanged}>
        <span class="summary-label">Size</span>
        <span class="old-val">{formatSize(pinnedResponse.bodySize)}</span>
        <span class="arrow">→</span>
        <span class="new-val">{formatSize(currentResponse.bodySize)}</span>
      </div>
      {#if !diff.hasChanges}
        <span class="no-changes">Bodies are identical</span>
      {/if}
    </div>

    <div class="diff-container">
      <div class="diff-header">
        <span class="diff-title">Pinned (old)</span>
        <span class="diff-title">Current (new)</span>
      </div>
      <div class="diff-body">
        <div class="diff-pane left">
          {#each diff.left as line}
            <div class="diff-line {line.type}">
              <span class="line-num">{line.lineNumber ?? ""}</span>
              <span class="line-content">{line.content}</span>
            </div>
          {/each}
        </div>
        <div class="diff-pane right">
          {#each diff.right as line}
            <div class="diff-line {line.type}">
              <span class="line-num">{line.lineNumber ?? ""}</span>
              <span class="line-content">{line.content}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</Modal>

<style>
  .diff-modal {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
    max-height: 70vh;
  }
  .summary {
    display: flex;
    gap: var(--sp-lg);
    align-items: center;
    flex-wrap: wrap;
    font-size: var(--fs-small);
  }
  .summary-item {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    color: var(--text-secondary);
  }
  .summary-item.changed {
    color: var(--color-warning, #f59e0b);
    font-weight: 600;
  }
  .summary-label {
    font-weight: 500;
  }
  .arrow {
    color: var(--text-tertiary);
  }
  .no-changes {
    font-size: var(--fs-small);
    color: var(--color-success, #22c55e);
    font-weight: 500;
  }
  .diff-container {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    overflow: hidden;
    flex: 1;
    min-height: 200px;
  }
  .diff-header {
    display: grid;
    grid-template-columns: 1fr 1fr;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
    padding: var(--sp-xs) var(--sp-sm);
  }
  .diff-title {
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-secondary);
  }
  .diff-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    overflow-y: auto;
    max-height: 50vh;
  }
  .diff-pane {
    font-family: var(--font-mono);
    font-size: var(--fs-caption);
    border-right: 1px solid var(--border-light);
  }
  .diff-pane:last-child {
    border-right: none;
  }
  .diff-line {
    display: flex;
    min-height: 1.4em;
    line-height: 1.4;
  }
  .diff-line.removed {
    background: var(--color-error-overlay);
  }
  .diff-line.added {
    background: var(--color-success-overlay);
  }
  .line-num {
    width: 3em;
    text-align: right;
    padding-right: var(--sp-xs);
    color: var(--text-tertiary);
    flex-shrink: 0;
    user-select: none;
  }
  .line-content {
    padding-right: var(--sp-sm);
    white-space: pre;
  }
</style>
