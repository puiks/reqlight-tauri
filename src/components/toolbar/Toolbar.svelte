<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import EnvironmentPicker from "../environment/EnvironmentPicker.svelte";

  let {
    onopenenvs,
    onimportcurl,
  }: {
    onopenenvs: () => void;
    onimportcurl: () => void;
  } = $props();

  function toggleSidebar() {
    appStore.sidebarVisible = !appStore.sidebarVisible;
  }

  function toggleTheme() {
    const html = document.documentElement;
    const current = html.getAttribute("data-theme");
    if (current === "dark") {
      html.removeAttribute("data-theme");
      localStorage.removeItem("theme");
    } else {
      html.setAttribute("data-theme", "dark");
      localStorage.setItem("theme", "dark");
    }
  }
</script>

<div class="toolbar" data-tauri-drag-region>
  <div class="left">
    <button class="icon-btn" onclick={toggleSidebar} title="Toggle Sidebar">
      ☰
    </button>
  </div>

  <div class="center" data-tauri-drag-region>
    <span class="app-title">Reqlight</span>
  </div>

  <div class="right">
    <EnvironmentPicker onmanage={onopenenvs} />
    <button class="icon-btn" onclick={onimportcurl} title="Import cURL">
      ⤓
    </button>
    <button class="icon-btn" onclick={toggleTheme} title="Toggle Theme">
      ◐
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 38px;
    padding: 0 var(--sp-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  .left,
  .right {
    display: flex;
    align-items: center;
    gap: var(--sp-sm);
  }
  .center {
    flex: 1;
    text-align: center;
  }
  .app-title {
    font-size: var(--fs-body);
    font-weight: 600;
    color: var(--text-secondary);
  }
  .icon-btn {
    font-size: var(--fs-callout);
    padding: 2px var(--sp-xs);
    color: var(--text-secondary);
  }
</style>
