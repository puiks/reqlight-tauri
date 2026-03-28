<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import type { AuthType, ApiKeyLocation } from "../../lib/types";
  import OAuth2Editor from "./OAuth2Editor.svelte";

  const authTypes: { value: AuthType; label: string }[] = [
    { value: "none", label: "No Auth" },
    { value: "bearerToken", label: "Bearer Token" },
    { value: "basicAuth", label: "Basic Auth" },
    { value: "apiKey", label: "API Key" },
    { value: "oauth2", label: "OAuth 2.0" },
  ];

  const apiKeyLocations: { value: ApiKeyLocation; label: string }[] = [
    { value: "header", label: "Header" },
    { value: "query", label: "Query Param" },
  ];

  let showToken = $state(false);
  let showPassword = $state(false);
  let showApiKeyValue = $state(false);

  function onAuthTypeChange(e: Event) {
    editorStore.authType = (e.target as HTMLSelectElement).value as AuthType;
    showToken = false;
    showPassword = false;
    showApiKeyValue = false;
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
      <div class="secret-field">
        <input
          id="bearer-token"
          type={showToken ? "text" : "password"}
          class="input"
          placeholder="Enter bearer token"
          bind:value={editorStore.bearerToken}
          oninput={() => editorStore.markDirty()}
        />
        <button
          class="eye-btn"
          title={showToken ? "Hide" : "Show"}
          onclick={() => (showToken = !showToken)}
        >{showToken ? "◉" : "○"}</button>
      </div>
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
      <div class="secret-field">
        <input
          id="basic-pass"
          type={showPassword ? "text" : "password"}
          class="input"
          placeholder="Password"
          bind:value={editorStore.basicPassword}
          oninput={() => editorStore.markDirty()}
        />
        <button
          class="eye-btn"
          title={showPassword ? "Hide" : "Show"}
          onclick={() => (showPassword = !showPassword)}
        >{showPassword ? "◉" : "○"}</button>
      </div>
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
      <div class="secret-field">
        <input
          id="apikey-value"
          type={showApiKeyValue ? "text" : "password"}
          class="input"
          placeholder="API key value"
          bind:value={editorStore.apiKeyValue}
          oninput={() => editorStore.markDirty()}
        />
        <button
          class="eye-btn"
          title={showApiKeyValue ? "Hide" : "Show"}
          onclick={() => (showApiKeyValue = !showApiKeyValue)}
        >{showApiKeyValue ? "◉" : "○"}</button>
      </div>
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
  {:else if editorStore.authType === "oauth2"}
    <OAuth2Editor />
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
  .secret-field {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .secret-field .input {
    flex: 1;
  }
  .eye-btn {
    font-size: var(--fs-body);
    color: var(--text-tertiary);
    padding: var(--sp-xs);
    flex-shrink: 0;
    line-height: 1;
  }
  .eye-btn:hover {
    color: var(--text-primary);
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
