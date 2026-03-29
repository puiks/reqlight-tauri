<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import type { EditorTab } from "../../lib/types";

  const tabs: { value: EditorTab; label: string }[] = [
    { value: "params", label: "Params" },
    { value: "headers", label: "Headers" },
    { value: "auth", label: "Auth" },
    { value: "body", label: "Body" },
    { value: "extract", label: "Extract" },
    { value: "assert", label: "Assert" },
    { value: "script", label: "Script" },
  ];
</script>

<div class="tab-bar">
  {#each tabs as tab}
    <button
      class:active={editorStore.activeEditorTab === tab.value}
      onclick={() => (editorStore.activeEditorTab = tab.value)}
    >
      {tab.label}
    </button>
  {/each}
</div>
