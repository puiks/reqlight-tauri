<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import { createEmptyMultipartField } from "../../lib/type-helpers";

  let { onchange }: { onchange?: () => void } = $props();

  function addField() {
    editorStore.multipartFields = [
      ...editorStore.multipartFields,
      createEmptyMultipartField(),
    ];
    onchange?.();
  }

  function removeField(index: number) {
    editorStore.multipartFields = editorStore.multipartFields.filter(
      (_, i) => i !== index,
    );
    if (editorStore.multipartFields.length === 0) {
      editorStore.multipartFields = [createEmptyMultipartField()];
    }
    onchange?.();
  }

  function toggleEnabled(index: number) {
    editorStore.multipartFields = editorStore.multipartFields.map((f, i) =>
      i === index ? { ...f, isEnabled: !f.isEnabled } : f,
    );
    onchange?.();
  }

  function handleFileSelect(index: number) {
    const input = document.createElement("input");
    input.type = "file";
    input.onchange = () => {
      const file = input.files?.[0];
      if (file) {
        // In Tauri, we get the file path via webkitRelativePath or name
        // For now, use the file name. The actual path resolution happens in Tauri.
        editorStore.multipartFields = editorStore.multipartFields.map((f, i) =>
          i === index ? { ...f, filePath: file.name, value: file.name } : f,
        );
        onchange?.();
      }
    };
    input.click();
  }

  function clearFile(index: number) {
    editorStore.multipartFields = editorStore.multipartFields.map((f, i) =>
      i === index ? { ...f, filePath: undefined, value: "" } : f,
    );
    onchange?.();
  }

  function isFileField(field: { filePath?: string }): boolean {
    return !!field.filePath;
  }
</script>

<div class="multipart-editor">
  <div class="field-list">
    {#each editorStore.multipartFields as field, index (field.id)}
      <div class="field-row" class:disabled={!field.isEnabled}>
        <button
          class="toggle-btn"
          class:enabled={field.isEnabled}
          onclick={() => toggleEnabled(index)}
          title={field.isEnabled ? "Disable" : "Enable"}
        >
          {field.isEnabled ? "✓" : "○"}
        </button>
        <input
          type="text"
          class="name-input"
          placeholder="Field name"
          bind:value={field.name}
          oninput={() => onchange?.()}
        />
        {#if isFileField(field)}
          <div class="file-display">
            <span class="file-name">{field.filePath}</span>
            <button class="clear-btn" onclick={() => clearFile(index)} title="Remove file">✕</button>
          </div>
        {:else}
          <input
            type="text"
            class="value-input"
            placeholder="Value"
            bind:value={field.value}
            oninput={() => onchange?.()}
          />
        {/if}
        <button
          class="file-btn"
          onclick={() => handleFileSelect(index)}
          title="Select file"
        >
          📎
        </button>
        <button
          class="remove-btn"
          onclick={() => removeField(index)}
          title="Remove field"
        >
          ✕
        </button>
      </div>
    {/each}
  </div>
  <button class="add-btn" onclick={addField}>+ Add Field</button>
</div>

<style>
  .multipart-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
  }
  .field-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: 2px 0;
  }
  .field-row.disabled {
    opacity: 0.5;
  }
  .toggle-btn {
    font-size: var(--fs-caption);
    width: 20px;
    height: 20px;
    padding: 0;
    text-align: center;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .toggle-btn.enabled {
    color: var(--color-success);
  }
  .name-input,
  .value-input {
    font-size: var(--fs-small);
    padding: 2px var(--sp-xs);
    flex: 1;
    min-width: 0;
  }
  .name-input {
    max-width: 140px;
  }
  .file-display {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: 2px var(--sp-xs);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    min-width: 0;
  }
  .file-name {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .clear-btn {
    font-size: var(--fs-caption);
    padding: 0 2px;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .file-btn {
    font-size: var(--fs-small);
    padding: 0 2px;
    flex-shrink: 0;
  }
  .remove-btn {
    font-size: var(--fs-caption);
    padding: 0 2px;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .add-btn {
    font-size: var(--fs-caption);
    color: var(--color-info);
    padding: var(--sp-xs) var(--sp-sm);
    align-self: flex-start;
  }
</style>
