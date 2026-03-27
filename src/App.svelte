<script lang="ts">
  import { onMount } from "svelte";
  import { appStore } from "./lib/stores/app.svelte";
  import { editorStore } from "./lib/stores/editor.svelte";
  import { initKeyboardShortcuts, registerShortcut } from "./lib/utils/keyboard";
  import { handleError } from "./lib/utils/errors";
  import Toolbar from "./components/toolbar/Toolbar.svelte";
  import MainLayout from "./components/layout/MainLayout.svelte";
  import Toast from "./components/shared/Toast.svelte";
  import ErrorFallback from "./components/shared/ErrorFallback.svelte";
  import EnvironmentEditor from "./components/environment/EnvironmentEditor.svelte";
  import CurlImportModal from "./components/toolbar/CurlImportModal.svelte";

  let showEnvEditor = $state(false);
  let showCurlImport = $state(false);

  function handleBoundaryError(error: unknown) {
    handleError(error, "boundary", { silent: true });
  }

  onMount(() => {
    // Load persisted state
    appStore.load().then(() => {
      // Restore last selected request
      if (appStore.selectedRequestId) {
        const request = appStore.selectedRequest;
        if (request) editorStore.loadFrom(request);
      }
    });

    // Catch unhandled promise rejections
    const handleRejection = (e: PromiseRejectionEvent) => {
      e.preventDefault();
      handleError(e.reason, "unhandledrejection");
    };
    window.addEventListener("unhandledrejection", handleRejection);

    // Register keyboard shortcuts
    const cleanup = initKeyboardShortcuts();

    // ⌘N - New Request
    const unsub1 = registerShortcut({
      key: "n",
      meta: true,
      handler: () => {
        const collectionId =
          appStore.selectedCollectionId ?? appStore.collections[0]?.id;
        if (collectionId) {
          const r = appStore.addRequest(collectionId);
          editorStore.loadFrom(r);
        }
      },
    });

    // ⌘⇧N - New Collection
    const unsub2 = registerShortcut({
      key: "n",
      meta: true,
      shift: true,
      handler: () => {
        appStore.addCollection();
      },
    });

    // ⌘E - Environment Editor
    const unsub3 = registerShortcut({
      key: "e",
      meta: true,
      handler: () => {
        showEnvEditor = !showEnvEditor;
      },
    });

    // ⌘Enter - Send Request (handled in URLBar too)
    const unsub4 = registerShortcut({
      key: "Enter",
      meta: true,
      handler: () => {
        editorStore.send();
      },
    });

    // Save on window close / unload
    const handleBeforeUnload = () => {
      editorStore.saveIfDirty();
      appStore.flushSave();
    };
    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      cleanup();
      unsub1();
      unsub2();
      unsub3();
      unsub4();
      window.removeEventListener("beforeunload", handleBeforeUnload);
      window.removeEventListener("unhandledrejection", handleRejection);
    };
  });
</script>

<div class="app">
  <Toolbar
    onopenenvs={() => (showEnvEditor = true)}
    onimportcurl={() => (showCurlImport = true)}
  />
  <svelte:boundary onerror={handleBoundaryError}>
    <MainLayout />
    {#snippet failed(error, reset)}
      <ErrorFallback {error} onreset={reset} />
    {/snippet}
  </svelte:boundary>
  <Toast />

  {#if showEnvEditor}
    <EnvironmentEditor onclose={() => (showEnvEditor = false)} />
  {/if}

  {#if showCurlImport}
    <CurlImportModal onclose={() => (showCurlImport = false)} />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
</style>
