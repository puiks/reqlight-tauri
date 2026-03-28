<script lang="ts">
  let {
    importType,
    jsonInput = $bindable(),
    isProcessing,
    errorMessage,
    onimport,
    onfileupload,
  }: {
    importType: "collection" | "environment" | "openapi" | "har";
    jsonInput: string;
    isProcessing: boolean;
    errorMessage: string | null;
    onimport: () => void;
    onfileupload: (e: Event) => void;
  } = $props();

  const acceptMap: Record<string, string> = {
    openapi: ".json,.yaml,.yml",
    har: ".har,.json",
    collection: ".json",
    environment: ".json",
  };

  function placeholderText(): string {
    if (importType === "openapi") return "Paste OpenAPI 3.x or Swagger 2.x spec (JSON or YAML)...";
    if (importType === "har") return "Paste HAR JSON here (exported from Chrome DevTools)...";
    return `Paste Postman ${importType === "collection" ? "Collection" : "Environment"} JSON here...`;
  }
</script>

<div class="import-section">
  <div class="file-upload">
    <label class="file-label">
      Choose file or paste content below
      <input type="file" accept={acceptMap[importType]} onchange={onfileupload} class="file-input" />
    </label>
  </div>
  <textarea
    class="json-textarea"
    placeholder={placeholderText()}
    bind:value={jsonInput}
    spellcheck="false"
  ></textarea>
  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}
  <button
    class="action-btn"
    onclick={onimport}
    disabled={isProcessing || !jsonInput.trim()}
  >
    {isProcessing ? "Importing..." : "Import"}
  </button>
</div>

<style>
  .import-section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
  }
  .file-upload {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
  }
  .file-label {
    cursor: pointer;
  }
  .file-input {
    display: block;
    margin-top: var(--sp-xs);
    font-size: var(--fs-caption);
  }
  .json-textarea {
    width: 100%;
    min-height: 150px;
    font-family: var(--font-mono);
    font-size: var(--fs-caption);
    padding: var(--sp-sm);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-input);
    resize: vertical;
  }
  .action-btn {
    background: var(--color-info);
    color: white;
    font-weight: 600;
    padding: var(--sp-sm) var(--sp-lg);
    border-radius: var(--radius-sm);
    align-self: flex-start;
  }
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    font-size: var(--fs-caption);
    color: var(--color-error);
    padding: var(--sp-xs);
  }
</style>
