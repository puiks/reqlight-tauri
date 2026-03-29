<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
</script>

<div class="script-editor">
  <label class="section">
    <span class="label-text">Pre-request Script</span>
    <span class="label-hint">Runs before the request is sent</span>
    <textarea
      class="editor-textarea"
      placeholder={`// Set environment variables\nrl.environment.set("timestamp", Date.now().toString());\n\n// Access crypto\nlet hash = crypto.sha256("my-data");`}
      bind:value={editorStore.preRequestScript}
      oninput={() => editorStore.markDirty()}
      spellcheck="false"
    ></textarea>
  </label>
  <label class="section">
    <span class="label-text">Test Script</span>
    <span class="label-hint">Runs after the response is received</span>
    <textarea
      class="editor-textarea"
      placeholder={`// Write test assertions\nrl.test("Status is 200", function() {\n  rl.expect(rl.response.status).toBe(200);\n});\n\nlet data = rl.response.json();\nrl.expect(data.id).toBeDefined();`}
      bind:value={editorStore.testScript}
      oninput={() => editorStore.markDirty()}
      spellcheck="false"
    ></textarea>
  </label>
</div>

<style>
  .script-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
    height: 100%;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
    flex: 1;
    min-height: 0;
  }
  .label-text {
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .label-hint {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
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
    tab-size: 2;
  }
  .editor-textarea:focus {
    border-color: var(--color-info);
    outline: none;
  }
</style>
