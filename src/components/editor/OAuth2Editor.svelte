<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import type { OAuthGrantType } from "../../lib/types";
  import { oauthClientCredentials, oauthAuthorizationCode, oauthRefreshToken } from "../../lib/commands";

  const grantTypes: { value: OAuthGrantType; label: string }[] = [
    { value: "client_credentials", label: "Client Credentials" },
    { value: "authorization_code", label: "Authorization Code" },
  ];

  let showOAuthSecret = $state(false);
  let oauthLoading = $state(false);
  let oauthError = $state<string | null>(null);

  async function fetchOAuthToken() {
    oauthLoading = true;
    oauthError = null;
    try {
      let result;
      if (editorStore.oauthGrantType === "client_credentials") {
        result = await oauthClientCredentials(
          editorStore.oauthTokenUrl,
          editorStore.oauthClientId,
          editorStore.oauthClientSecret,
          editorStore.oauthScopes,
        );
      } else {
        result = await oauthAuthorizationCode({
          authUrl: editorStore.oauthAuthUrl,
          tokenUrl: editorStore.oauthTokenUrl,
          clientId: editorStore.oauthClientId,
          clientSecret: editorStore.oauthClientSecret,
          scopes: editorStore.oauthScopes,
        });
      }
      editorStore.oauthAccessToken = result.accessToken;
      editorStore.oauthRefreshToken = result.refreshToken;
      if (result.expiresIn) {
        const expiry = new Date(Date.now() + result.expiresIn * 1000);
        editorStore.oauthTokenExpiry = expiry.toISOString();
      }
      editorStore.markDirty();
    } catch (e) {
      oauthError = e instanceof Error ? e.message : String(e);
    } finally {
      oauthLoading = false;
    }
  }

  async function refreshOAuthTokenAction() {
    if (!editorStore.oauthRefreshToken) return;
    oauthLoading = true;
    oauthError = null;
    try {
      const result = await oauthRefreshToken(
        editorStore.oauthTokenUrl,
        editorStore.oauthRefreshToken,
        editorStore.oauthClientId,
        editorStore.oauthClientSecret,
      );
      editorStore.oauthAccessToken = result.accessToken;
      if (result.refreshToken) editorStore.oauthRefreshToken = result.refreshToken;
      if (result.expiresIn) {
        const expiry = new Date(Date.now() + result.expiresIn * 1000);
        editorStore.oauthTokenExpiry = expiry.toISOString();
      }
      editorStore.markDirty();
    } catch (e) {
      oauthError = e instanceof Error ? e.message : String(e);
    } finally {
      oauthLoading = false;
    }
  }
</script>

<div class="oauth-section">
  <div class="auth-fields">
    <label class="label" for="oauth-grant">Grant Type</label>
    <select
      id="oauth-grant"
      class="select"
      value={editorStore.oauthGrantType}
      onchange={(e) => {
        editorStore.oauthGrantType = (e.target as HTMLSelectElement).value as OAuthGrantType;
        editorStore.markDirty();
      }}
    >
      {#each grantTypes as g}
        <option value={g.value}>{g.label}</option>
      {/each}
    </select>

    {#if editorStore.oauthGrantType === "authorization_code"}
      <label class="label" for="oauth-auth-url">Auth URL</label>
      <input
        id="oauth-auth-url"
        type="text"
        class="input"
        placeholder="https://provider.com/authorize"
        bind:value={editorStore.oauthAuthUrl}
        oninput={() => editorStore.markDirty()}
      />
    {/if}

    <label class="label" for="oauth-token-url">Token URL</label>
    <input
      id="oauth-token-url"
      type="text"
      class="input"
      placeholder="https://provider.com/token"
      bind:value={editorStore.oauthTokenUrl}
      oninput={() => editorStore.markDirty()}
    />

    <label class="label" for="oauth-client-id">Client ID</label>
    <input
      id="oauth-client-id"
      type="text"
      class="input"
      placeholder="Client ID"
      bind:value={editorStore.oauthClientId}
      oninput={() => editorStore.markDirty()}
    />

    <label class="label" for="oauth-client-secret">Client Secret</label>
    <div class="secret-field">
      <input
        id="oauth-client-secret"
        type={showOAuthSecret ? "text" : "password"}
        class="input"
        placeholder="Client Secret"
        bind:value={editorStore.oauthClientSecret}
        oninput={() => editorStore.markDirty()}
      />
      <button
        class="eye-btn"
        title={showOAuthSecret ? "Hide" : "Show"}
        onclick={() => (showOAuthSecret = !showOAuthSecret)}
      >{showOAuthSecret ? "◉" : "○"}</button>
    </div>

    <label class="label" for="oauth-scopes">Scopes</label>
    <input
      id="oauth-scopes"
      type="text"
      class="input"
      placeholder="read write (space-separated)"
      bind:value={editorStore.oauthScopes}
      oninput={() => editorStore.markDirty()}
    />
  </div>

  <div class="oauth-actions">
    <button
      class="btn btn-primary"
      disabled={oauthLoading || !editorStore.oauthTokenUrl}
      onclick={fetchOAuthToken}
    >
      {oauthLoading ? "Fetching…" : "Get New Token"}
    </button>
    {#if editorStore.oauthRefreshToken}
      <button
        class="btn btn-secondary"
        disabled={oauthLoading}
        onclick={refreshOAuthTokenAction}
      >
        Refresh Token
      </button>
    {/if}
  </div>

  {#if oauthError}
    <p class="oauth-error">{oauthError}</p>
  {/if}

  {#if editorStore.oauthAccessToken}
    <div class="oauth-token-display">
      <span class="label">Access Token</span>
      <code class="token-value">{editorStore.oauthAccessToken.slice(0, 40)}…</code>
      {#if editorStore.oauthTokenExpiry}
        <span class="token-expiry">Expires: {new Date(editorStore.oauthTokenExpiry).toLocaleString()}</span>
      {/if}
    </div>
  {/if}
</div>

<style>
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
  .oauth-section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
  }
  .oauth-actions {
    display: flex;
    gap: var(--sp-sm);
  }
  .btn {
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-md);
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .oauth-error {
    font-size: var(--fs-small);
    color: var(--color-error, #ef4444);
  }
  .oauth-token-display {
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
    padding: var(--sp-sm);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
  }
  .token-value {
    font-size: var(--fs-small);
    font-family: var(--font-mono);
    word-break: break-all;
    color: var(--text-secondary);
  }
  .token-expiry {
    font-size: var(--fs-xsmall, 11px);
    color: var(--text-tertiary);
  }
</style>
