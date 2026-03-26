<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";

  let { onmanage }: { onmanage: () => void } = $props();
</script>

<div class="env-picker">
  <select
    class="env-select"
    value={appStore.activeEnvironmentId ?? ""}
    onchange={(e) => {
      const val = e.currentTarget.value;
      appStore.setActiveEnvironment(val || null);
    }}
  >
    <option value="">No Environment</option>
    {#each appStore.environments as env}
      <option value={env.id}>{env.name}</option>
    {/each}
  </select>
  <button class="manage-btn" onclick={onmanage} title="Manage Environments (⌘E)">
    ⚙
  </button>
</div>

<style>
  .env-picker {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .env-select {
    font-size: var(--fs-small);
    background: var(--bg-input);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 2px var(--sp-sm);
    color: var(--text-primary);
    cursor: pointer;
  }
  .manage-btn {
    font-size: var(--fs-body);
    color: var(--text-secondary);
    padding: 2px var(--sp-xs);
  }
</style>
