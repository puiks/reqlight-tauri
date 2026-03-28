<script lang="ts">
  import Modal from "../shared/Modal.svelte";
  import { generateCode } from "../../lib/commands";
  import { editorStore } from "../../lib/stores/editor.svelte";
  import { environmentStore } from "../../lib/stores/environment.svelte";
  import type { CodegenLanguage } from "../../lib/types";

  let { onclose }: { onclose: () => void } = $props();

  const languages: { value: CodegenLanguage; label: string }[] = [
    { value: "javascript-fetch", label: "Fetch" },
    { value: "javascript-axios", label: "Axios" },
    { value: "python-requests", label: "Python" },
    { value: "curl", label: "cURL" },
  ];

  let selectedLanguage = $state<CodegenLanguage>("javascript-fetch");
  let code = $state("");
  let isLoading = $state(false);
  let copied = $state(false);

  async function generate() {
    const request = editorStore.toSavedRequest();
    if (!request) return;
    isLoading = true;
    try {
      code = await generateCode(request, selectedLanguage, environmentStore.activeEnvironment);
    } catch (e) {
      code = `Error: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      isLoading = false;
    }
  }

  async function copyToClipboard() {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  // Generate on mount and language change
  $effect(() => {
    // Track selectedLanguage to re-run
    void selectedLanguage;
    generate();
  });
</script>

<Modal title="Generate Code" {onclose}>
  <div class="codegen">
    <div class="lang-tabs">
      {#each languages as lang}
        <button
          class="lang-tab"
          class:active={selectedLanguage === lang.value}
          onclick={() => (selectedLanguage = lang.value)}
        >
          {lang.label}
        </button>
      {/each}
    </div>

    <div class="code-area">
      {#if isLoading}
        <div class="loading">Generating...</div>
      {:else}
        <pre class="code-block">{code}</pre>
      {/if}
    </div>

    <div class="actions">
      <button class="copy-btn" onclick={copyToClipboard} disabled={!code || isLoading}>
        {copied ? "Copied!" : "Copy to Clipboard"}
      </button>
    </div>
  </div>
</Modal>

<style>
  .codegen {
    min-width: 500px;
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
  }
  .lang-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-color);
  }
  .lang-tab {
    padding: var(--sp-sm) var(--sp-lg);
    font-size: var(--fs-small);
    font-weight: 500;
    color: var(--text-secondary);
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: transparent;
  }
  .lang-tab:hover {
    color: var(--text-primary);
  }
  .lang-tab.active {
    color: var(--color-info);
    border-bottom-color: var(--color-info);
    font-weight: 600;
  }
  .code-area {
    max-height: 400px;
    overflow: auto;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
  }
  .code-block {
    margin: 0;
    padding: var(--sp-md);
    font-family: var(--font-mono, monospace);
    font-size: var(--fs-small);
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-primary);
    line-height: 1.5;
  }
  .loading {
    padding: var(--sp-lg);
    text-align: center;
    color: var(--text-secondary);
    font-size: var(--fs-small);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .copy-btn {
    padding: var(--sp-sm) var(--sp-lg);
    font-size: var(--fs-small);
    font-weight: 500;
    background: var(--color-info);
    color: white;
    border-radius: var(--radius-sm);
  }
  .copy-btn:hover:not(:disabled) {
    opacity: 0.9;
  }
  .copy-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
