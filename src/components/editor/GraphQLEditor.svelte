<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
</script>

<div class="graphql-editor">
  <label class="section">
    <span class="label-text">Query</span>
    <textarea
      class="editor-textarea query"
      placeholder={"query { users { id name email } }"}
      bind:value={editorStore.graphqlQuery}
      oninput={() => editorStore.markDirty()}
      spellcheck="false"
    ></textarea>
  </label>
  <label class="section">
    <span class="label-text">Variables (JSON)</span>
    <textarea
      class="editor-textarea variables"
      placeholder={'{ "id": 1 }'}
      bind:value={editorStore.graphqlVariables}
      oninput={() => editorStore.markDirty()}
      spellcheck="false"
    ></textarea>
  </label>
</div>

<style>
  .graphql-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
    height: 100%;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
  }
  .section:first-child {
    flex: 2;
  }
  .section:last-child {
    flex: 1;
  }
  .label-text {
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .editor-textarea {
    flex: 1;
    min-height: 80px;
    font-family: var(--font-mono, monospace);
    font-size: var(--fs-small);
    padding: var(--sp-sm);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    resize: vertical;
    line-height: 1.5;
  }
  .editor-textarea:focus {
    border-color: var(--color-info);
    outline: none;
  }
</style>
