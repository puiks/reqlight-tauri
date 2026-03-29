<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import { createEmptyExtractionRule } from "../../lib/type-helpers";

  function handleInput(index: number) {
    editorStore.markDirty();
    autoAddRow(index);
  }

  function autoAddRow(index: number) {
    const rules = editorStore.extractionRules;
    if (index === rules.length - 1) {
      const last = rules[index];
      if (last.variableName || last.jsonPath) {
        editorStore.extractionRules = [...rules, createEmptyExtractionRule()];
      }
    }
  }

  function removeRule(index: number) {
    editorStore.extractionRules = editorStore.extractionRules.filter((_, i) => i !== index);
    editorStore.markDirty();
  }

  function toggleEnabled(index: number) {
    editorStore.extractionRules[index].isEnabled = !editorStore.extractionRules[index].isEnabled;
    editorStore.markDirty();
  }
</script>

<div class="extraction-editor">
  <div class="header-row">
    <span class="col-check"></span>
    <span class="col-var">Variable Name</span>
    <span class="col-path">JSONPath Expression</span>
    <span class="col-action"></span>
  </div>

  {#each editorStore.extractionRules as rule, index (rule.id)}
    <div class="row" class:disabled={!rule.isEnabled}>
      <span class="col-check">
        <input
          type="checkbox"
          checked={rule.isEnabled}
          onchange={() => toggleEnabled(index)}
        />
      </span>
      <span class="col-var">
        <input
          type="text"
          placeholder="token"
          bind:value={rule.variableName}
          oninput={() => handleInput(index)}
          disabled={!rule.isEnabled}
        />
      </span>
      <span class="col-path">
        <input
          type="text"
          placeholder="$.data.access_token"
          bind:value={rule.jsonPath}
          oninput={() => handleInput(index)}
          disabled={!rule.isEnabled}
        />
      </span>
      <span class="col-action">
        {#if rule.variableName || rule.jsonPath}
          <button class="delete-btn" onclick={() => removeRule(index)}>×</button>
        {/if}
      </span>
    </div>
  {/each}

  <p class="hint">
    Extract values from JSON responses into your active environment.
    Use dot notation: <code>$.data.token</code>, <code>$.items[0].id</code>
  </p>
</div>

<style>
  .extraction-editor {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .header-row {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: var(--sp-xs) 0;
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: 2px 0;
  }
  .row.disabled {
    opacity: 0.5;
  }
  .col-check {
    width: 30px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .col-check input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }
  .col-var {
    flex: 1;
  }
  .col-path {
    flex: 1.5;
  }
  .col-action {
    width: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .row input[type="text"] {
    width: 100%;
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-sm);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  .row input[type="text"]:focus {
    border-color: var(--color-info);
    outline: none;
  }
  .delete-btn {
    font-size: var(--fs-body);
    color: var(--text-tertiary);
    padding: 0;
    line-height: 1;
    background: transparent;
    border: none;
    cursor: pointer;
  }
  .delete-btn:hover {
    color: var(--color-error);
  }
  .hint {
    margin-top: var(--sp-md);
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    line-height: 1.4;
  }
  .hint code {
    background: var(--bg-tertiary);
    padding: 1px 4px;
    border-radius: 3px;
    font-family: var(--font-mono, monospace);
    font-size: var(--fs-caption);
  }
</style>
