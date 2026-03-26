<script lang="ts">
  import { environmentStore } from "../../lib/stores/environment.svelte";
  import KeyValueEditor from "../editor/KeyValueEditor.svelte";
  import Modal from "../shared/Modal.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let selectedEnvId = $state<string | null>(
    environmentStore.environments[0]?.id ?? null,
  );

  const selectedEnv = $derived(
    environmentStore.environments.find((e) => e.id === selectedEnvId),
  );

  function addEnvironment() {
    const env = environmentStore.addEnvironment();
    selectedEnvId = env.id;
  }

  function deleteEnvironment() {
    if (!selectedEnvId) return;
    environmentStore.deleteEnvironment(selectedEnvId);
    selectedEnvId = environmentStore.environments[0]?.id ?? null;
  }

  function handleNameChange(e: Event) {
    if (!selectedEnv) return;
    const name = (e.target as HTMLInputElement).value;
    environmentStore.updateEnvironment({ ...selectedEnv, name });
  }

  function handleVarsChange() {
    if (!selectedEnv) return;
    environmentStore.updateEnvironment(selectedEnv);
  }
</script>

<Modal title="Environments" {onclose}>
  <div class="env-editor">
    <div class="env-list">
      <div class="list-header">
        <button class="add-btn" onclick={addEnvironment}>+ Add</button>
      </div>
      {#each environmentStore.environments as env (env.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="env-item"
          class:selected={selectedEnvId === env.id}
          onclick={() => (selectedEnvId = env.id)}
        >
          {env.name}
        </div>
      {/each}
      {#if environmentStore.environments.length === 0}
        <div class="no-envs">No environments yet.</div>
      {/if}
    </div>

    <div class="env-detail">
      {#if selectedEnv}
        <div class="detail-header">
          <input
            type="text"
            class="env-name-input"
            value={selectedEnv.name}
            oninput={handleNameChange}
          />
          <button class="delete-btn" onclick={deleteEnvironment}>Delete</button>
        </div>
        <div class="vars-section">
          <KeyValueEditor
            bind:pairs={selectedEnv.variables}
            showSecret={true}
            onchange={handleVarsChange}
          />
        </div>
      {:else}
        <div class="no-selection">Select an environment to edit.</div>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .env-editor {
    display: flex;
    gap: 0;
    min-height: 320px;
  }
  .env-list {
    width: 200px;
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
  }
  .list-header {
    padding: var(--sp-sm);
    border-bottom: 1px solid var(--border-light);
  }
  .add-btn {
    font-size: var(--fs-small);
    color: var(--color-info);
    font-weight: 600;
  }
  .env-item {
    padding: var(--sp-sm) var(--sp-md);
    font-size: var(--fs-small);
    cursor: pointer;
    border-radius: var(--radius-sm);
    margin: 1px var(--sp-xs);
  }
  .env-item:hover {
    background: var(--bg-hover);
  }
  .env-item.selected {
    background: var(--bg-selected);
    color: var(--color-info);
    font-weight: 600;
  }
  .no-envs {
    padding: var(--sp-md);
    font-size: var(--fs-small);
    color: var(--text-tertiary);
    text-align: center;
  }
  .env-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--sp-sm);
    min-width: 0;
  }
  .detail-header {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
    margin-bottom: var(--sp-md);
  }
  .env-name-input {
    flex: 1;
    font-size: var(--fs-callout);
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-sm);
  }
  .delete-btn {
    font-size: var(--fs-small);
    color: var(--color-error);
    padding: var(--sp-xs) var(--sp-sm);
  }
  .vars-section {
    flex: 1;
    overflow: auto;
  }
  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: var(--fs-small);
  }
</style>
