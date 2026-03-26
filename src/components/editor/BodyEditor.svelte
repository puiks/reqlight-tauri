<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import KeyValueEditor from "./KeyValueEditor.svelte";
  import { validateJson } from "../../lib/utils/json-highlighter";
  import type { BodyType } from "../../lib/types";

  const bodyTypes: { value: BodyType; label: string }[] = [
    { value: "none", label: "None" },
    { value: "json", label: "JSON" },
    { value: "formData", label: "Form Data" },
    { value: "rawText", label: "Raw Text" },
  ];

  const jsonError = $derived(
    editorStore.bodyType === "json"
      ? validateJson(editorStore.jsonBody)
      : null,
  );
</script>

<div class="body-editor">
  <div class="type-selector">
    {#each bodyTypes as bt}
      <button
        class="type-btn"
        class:active={editorStore.bodyType === bt.value}
        onclick={() => {
          editorStore.bodyType = bt.value;
          editorStore.markDirty();
        }}
      >
        {bt.label}
      </button>
    {/each}
  </div>

  <div class="body-content">
    {#if editorStore.bodyType === "none"}
      <div class="empty-body">
        <span class="text-secondary">This request has no body.</span>
      </div>
    {:else if editorStore.bodyType === "json"}
      <div class="json-editor">
        <textarea
          class="json-input"
          placeholder={'{"key": "value"}'}
          bind:value={editorStore.jsonBody}
          oninput={() => editorStore.markDirty()}
          spellcheck="false"
        ></textarea>
        {#if jsonError}
          <div class="json-error">{jsonError}</div>
        {/if}
      </div>
    {:else if editorStore.bodyType === "formData"}
      <KeyValueEditor
        bind:pairs={editorStore.formPairs}
        onchange={() => editorStore.markDirty()}
      />
    {:else if editorStore.bodyType === "rawText"}
      <textarea
        class="raw-input"
        placeholder="Enter raw text..."
        bind:value={editorStore.rawBody}
        oninput={() => editorStore.markDirty()}
        spellcheck="false"
      ></textarea>
    {/if}
  </div>
</div>

<style>
  .body-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .type-selector {
    display: flex;
    gap: 2px;
    padding: var(--sp-sm) 0;
    border-bottom: 1px solid var(--border-light);
  }
  .type-btn {
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-md);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }
  .type-btn.active {
    background: var(--bg-selected);
    color: var(--color-info);
    font-weight: 600;
  }
  .body-content {
    flex: 1;
    overflow: auto;
    padding-top: var(--sp-sm);
  }
  .empty-body {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: var(--fs-small);
  }
  .json-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .json-input,
  .raw-input {
    flex: 1;
    width: 100%;
    min-height: 120px;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    border: none;
    background: transparent;
    resize: none;
    padding: var(--sp-sm);
    line-height: 1.5;
  }
  .json-input:focus,
  .raw-input:focus {
    box-shadow: none;
  }
  .json-error {
    font-size: var(--fs-caption);
    color: var(--color-error);
    padding: var(--sp-xs) var(--sp-sm);
    border-top: 1px solid var(--border-light);
  }
  .text-secondary {
    color: var(--text-secondary);
  }
</style>
