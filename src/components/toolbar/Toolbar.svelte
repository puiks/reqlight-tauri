<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import EnvironmentPicker from "../environment/EnvironmentPicker.svelte";

  let {
    onopenenvs,
  }: {
    onopenenvs: () => void;
  } = $props();

  function toggleSidebar() {
    appStore.sidebarVisible = !appStore.sidebarVisible;
  }

  let isDark = $state(false);

  $effect(() => {
    // Initialize from saved preference or system
    const saved = localStorage.getItem("theme");
    if (saved) {
      isDark = saved === "dark";
    } else {
      isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  });

  function toggleTheme() {
    isDark = !isDark;
    const theme = isDark ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
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
    <button class="icon-btn" onclick={toggleTheme} title="Toggle Theme">
      {isDark ? '☀' : '☾'}
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
    padding: var(--sp-xs) var(--sp-sm);
    color: var(--text-secondary);
  }
  .icon-btn:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
  }
</style>
