<script lang="ts">
  import Modal from "../shared/Modal.svelte";
  import ImportSection from "./ImportSection.svelte";
  import { appStore } from "../../lib/stores/app.svelte";
  import { environmentStore } from "../../lib/stores/environment.svelte";
  import { toastStore } from "../../lib/stores/toast.svelte";
  import {
    importPostmanCollection,
    exportPostmanCollection,
    importPostmanEnvironment,
    exportPostmanEnvironment,
    importOpenapi,
    importHar,
  } from "../../lib/commands";

  let { onclose }: { onclose: () => void } = $props();

  let activeTab = $state<"import" | "export">("import");
  let importType = $state<"collection" | "environment" | "openapi" | "har">("collection");
  let jsonInput = $state("");
  let isProcessing = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleImport() {
    if (!jsonInput.trim()) return;
    isProcessing = true;
    errorMessage = null;
    try {
      if (importType === "openapi") {
        const collections = await importOpenapi(jsonInput);
        appStore.collections = [...appStore.collections, ...collections];
        appStore.scheduleSave();
        const count = collections.reduce((sum, c) => sum + c.requests.length, 0);
        toastStore.show(`Imported ${collections.length} collection(s) with ${count} requests`);
      } else if (importType === "har") {
        const collection = await importHar(jsonInput);
        appStore.collections = [...appStore.collections, collection];
        appStore.scheduleSave();
        toastStore.show(`Imported ${collection.requests.length} requests from HAR`);
      } else if (importType === "collection") {
        const collection = await importPostmanCollection(jsonInput);
        appStore.collections = [...appStore.collections, collection];
        appStore.scheduleSave();
        toastStore.show(`Collection "${collection.name}" imported`);
      } else {
        const env = await importPostmanEnvironment(jsonInput);
        environmentStore.environments = [...environmentStore.environments, env];
        toastStore.show(`Environment "${env.name}" imported`);
      }
      onclose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  let selectedCollectionId = $state<string | null>(null);
  let selectedEnvironmentId = $state<string | null>(null);
  let exportResult = $state<string | null>(null);

  async function handleExport() {
    isProcessing = true;
    errorMessage = null;
    try {
      if (importType === "collection" && selectedCollectionId) {
        const collection = appStore.collections.find((c) => c.id === selectedCollectionId);
        if (collection) {
          exportResult = await exportPostmanCollection(collection);
          toastStore.show("Collection exported as Postman JSON");
        }
      } else if (importType === "environment" && selectedEnvironmentId) {
        const env = environmentStore.environments.find((e) => e.id === selectedEnvironmentId);
        if (env) {
          exportResult = await exportPostmanEnvironment(env);
          toastStore.show("Environment exported as Postman JSON");
        }
      }
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  function handleCopyExport() {
    if (exportResult) {
      navigator.clipboard.writeText(exportResult);
      toastStore.show("Copied to clipboard");
    }
  }

  function handleDownloadExport() {
    if (!exportResult) return;
    const name = importType === "collection" ? "collection" : "environment";
    const blob = new Blob([exportResult], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${name}.postman.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleFileUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      jsonInput = reader.result as string;
    };
    reader.readAsText(file);
  }
</script>

<Modal title="Import / Export" {onclose}>
  <div class="io-modal">
    <div class="tab-bar">
      <button class="tab" class:active={activeTab === "import"} onclick={() => { activeTab = "import"; exportResult = null; }}>
        Import
      </button>
      <button class="tab" class:active={activeTab === "export"} onclick={() => { activeTab = "export"; errorMessage = null; if (importType === "openapi" || importType === "har") importType = "collection"; }}>
        Export
      </button>
    </div>

    <div class="type-bar">
      <button class="type-btn" class:active={importType === "collection"} onclick={() => (importType = "collection")}>
        Postman Collection
      </button>
      <button class="type-btn" class:active={importType === "environment"} onclick={() => (importType = "environment")}>
        Postman Environment
      </button>
      {#if activeTab === "import"}
        <button class="type-btn" class:active={importType === "openapi"} onclick={() => (importType = "openapi")}>
          OpenAPI / Swagger
        </button>
        <button class="type-btn" class:active={importType === "har"} onclick={() => (importType = "har")}>
          HAR
        </button>
      {/if}
    </div>

    {#if activeTab === "import"}
      <ImportSection
        {importType}
        bind:jsonInput
        {isProcessing}
        {errorMessage}
        onimport={handleImport}
        onfileupload={handleFileUpload}
      />
    {:else}
      <div class="export-section">
        {#if importType === "collection"}
          <select class="select" bind:value={selectedCollectionId}>
            <option value={null}>Select a collection...</option>
            {#each appStore.collections as c}
              <option value={c.id}>{c.name} ({c.requests.length} requests)</option>
            {/each}
          </select>
        {:else}
          <select class="select" bind:value={selectedEnvironmentId}>
            <option value={null}>Select an environment...</option>
            {#each environmentStore.environments as e}
              <option value={e.id}>{e.name}</option>
            {/each}
          </select>
        {/if}
        <button class="action-btn" onclick={handleExport} disabled={isProcessing}>
          {isProcessing ? "Exporting..." : "Export as Postman JSON"}
        </button>
        {#if exportResult}
          <textarea class="json-textarea export-result" readonly value={exportResult}></textarea>
          <div class="export-actions">
            <button class="small-btn" onclick={handleCopyExport}>Copy</button>
            <button class="small-btn" onclick={handleDownloadExport}>Download</button>
          </div>
        {/if}
        {#if errorMessage}
          <div class="error">{errorMessage}</div>
        {/if}
      </div>
    {/if}
  </div>
</Modal>

<style>
  .io-modal {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
    min-width: 400px;
  }
  .tab-bar {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border-light);
    padding-bottom: var(--sp-xs);
  }
  .tab {
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-lg);
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    color: var(--text-secondary);
  }
  .tab.active {
    color: var(--color-info);
    font-weight: 600;
    border-bottom: 2px solid var(--color-info);
  }
  .type-bar {
    display: flex;
    gap: var(--sp-xs);
  }
  .type-btn {
    font-size: var(--fs-caption);
    padding: var(--sp-xs) var(--sp-md);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }
  .type-btn.active {
    background: var(--bg-selected);
    color: var(--color-info);
    font-weight: 600;
  }
  .export-section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
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
  .export-result {
    min-height: 100px;
    background: var(--bg-editor);
  }
  .select {
    width: 100%;
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-sm);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-input);
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
  .export-actions {
    display: flex;
    gap: var(--sp-sm);
  }
  .small-btn {
    font-size: var(--fs-caption);
    padding: var(--sp-xs) var(--sp-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
  }
  .error {
    font-size: var(--fs-caption);
    color: var(--color-error);
    padding: var(--sp-xs);
  }
</style>
