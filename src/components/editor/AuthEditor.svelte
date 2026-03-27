<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import type { AuthType, ApiKeyLocation } from "../../lib/types";

  const authTypes: { value: AuthType; label: string }[] = [
    { value: "none", label: "No Auth" },
    { value: "bearerToken", label: "Bearer Token" },
    { value: "basicAuth", label: "Basic Auth" },
    { value: "apiKey", label: "API Key" },
  ];

  const apiKeyLocations: { value: ApiKeyLocation; label: string }[] = [
    { value: "header", label: "Header" },
    { value: "query", label: "Query Param" },
  ];

  function onAuthTypeChange(e: Event) {
    editorStore.authType = (e.target as HTMLSelectElement).value as AuthType;
    editorStore.markDirty();
  }
</script>

<div class="auth-editor">
  <div class="auth-type-row">
    <label class="label" for="auth-type">Type</label>
    <select id="auth-type" class="select" value={editorStore.authType} onchange={onAuthTypeChange}>
      {#each authTypes as t}
        <option value={t.value}>{t.label}</option>
      {/each}
    </select>
  </div>

  {#if editorStore.authType === "bearerToken"}
    <div class="auth-fields">
      <label class="label" for="bearer-token">Token</label>
      <input
        id="bearer-token"
        type="password"
        class="input"
        placeholder="Enter bearer token"
        bind:value={editorStore.bearerToken}
        oninput={() => editorStore.markDirty()}
      />
    </div>
  {:else if editorStore.authType === "basicAuth"}
    <div class="auth-fields">
      <label class="label" for="basic-user">Username</label>
      <input
        id="basic-user"
        type="text"
        class="input"
        placeholder="Username"
        bind:value={editorStore.basicUsername}
        oninput={() => editorStore.markDirty()}
      />
      <label class="label" for="basic-pass">Password</label>
      <input
        id="basic-pass"
        type="password"
        class="input"
        placeholder="Password"
        bind:value={editorStore.basicPassword}
        oninput={() => editorStore.markDirty()}
      />
    </div>
  {:else if editorStore.authType === "apiKey"}
    <div class="auth-fields">
      <label class="label" for="apikey-key">Key</label>
      <input
        id="apikey-key"
        type="text"
        class="input"
        placeholder="e.g. X-API-Key"
        bind:value={editorStore.apiKeyKey}
        oninput={() => editorStore.markDirty()}
      />
      <label class="label" for="apikey-value">Value</label>
      <input
        id="apikey-value"
        type="password"
        class="input"
        placeholder="API key value"
        bind:value={editorStore.apiKeyValue}
        oninput={() => editorStore.markDirty()}
      />
      <label class="label" for="apikey-loc">Add to</label>
      <select
        id="apikey-loc"
        class="select"
        value={editorStore.apiKeyLocation}
        onchange={(e) => {
          editorStore.apiKeyLocation = (e.target as HTMLSelectElement).value as ApiKeyLocation;
          editorStore.markDirty();
        }}
      >
        {#each apiKeyLocations as loc}
          <option value={loc.value}>{loc.label}</option>
        {/each}
      </select>
    </div>
  {:else}
    <p class="no-auth-hint">This request does not use any authentication.</p>
  {/if}
</div>

<style>
  .auth-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
  }
  .auth-type-row {
    display: flex;
    align-items: center;
    gap: var(--sp-md);
  }
  .auth-fields {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--sp-sm) var(--sp-md);
    align-items: center;
  }
  .label {
    font-size: var(--fs-small);
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .input {
    width: 100%;
    font-size: var(--fs-small);
    font-family: var(--font-mono);
  }
  .select {
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-sm);
    min-width: 140px;
  }
  .no-auth-hint {
    font-size: var(--fs-small);
    color: var(--text-tertiary);
  }
</style>
