<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import type { EditorTab } from "../../lib/types";

  const tabs: { value: EditorTab; label: string }[] = [
    { value: "params", label: "Params" },
    { value: "headers", label: "Headers" },
    { value: "auth", label: "Auth" },
    { value: "body", label: "Body" },
    { value: "extract", label: "Extract" },
  ];
</script>

<div class="tabs">
  {#each tabs as tab}
    <button
      class="tab"
      class:active={editorStore.activeEditorTab === tab.value}
      onclick={() => (editorStore.activeEditorTab = tab.value)}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-color);
  }
  .tab {
    padding: var(--sp-sm) var(--sp-lg);
    font-size: var(--fs-small);
    font-weight: 500;
    color: var(--text-secondary);
    border-bottom: 2px solid transparent;
    border-radius: 0;
    transition: color 0.1s, border-color 0.1s;
  }
  .tab:hover {
    color: var(--text-primary);
    background: transparent;
  }
  .tab.active {
    color: var(--color-info);
    border-bottom-color: var(--color-info);
    font-weight: 600;
  }
</style>
