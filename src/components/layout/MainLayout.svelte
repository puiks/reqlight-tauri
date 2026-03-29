<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import Sidebar from "../sidebar/Sidebar.svelte";
  import RequestEditor from "../editor/RequestEditor.svelte";
  import ResponseView from "../response/ResponseView.svelte";
  import type { RequestCollection } from "../../lib/types";

  let {
    onimportcurl,
    oncollectionio,
    ongeneratecode,
    oncompare,
    onruncollection,
  }: {
    onimportcurl: () => void;
    oncollectionio: () => void;
    ongeneratecode?: () => void;
    oncompare?: () => void;
    onruncollection?: (collection: RequestCollection) => void;
  } = $props();

  let sidebarWidth = $state(220);
  let editorRatio = $state(0.7);
  let isDraggingSidebar = $state(false);
  let isDraggingSplit = $state(false);
  let containerEl: HTMLDivElement;

  function startSidebarDrag(e: MouseEvent) {
    isDraggingSidebar = true;
    e.preventDefault();
  }

  function startSplitDrag(e: MouseEvent) {
    isDraggingSplit = true;
    e.preventDefault();
  }

  let rafPending = false;
  function handleMouseMove(e: MouseEvent) {
    if (!isDraggingSidebar && !isDraggingSplit) return;
    if (rafPending) return;
    rafPending = true;
    const clientX = e.clientX;
    requestAnimationFrame(() => {
      rafPending = false;
      if (isDraggingSidebar) {
        sidebarWidth = Math.max(200, Math.min(320, clientX));
      }
      if (isDraggingSplit && containerEl) {
        const rect = containerEl.getBoundingClientRect();
        const sidebarOffset = appStore.sidebarVisible ? sidebarWidth + 4 : 0;
        const availableWidth = rect.width - sidebarOffset;
        const relX = clientX - rect.left - sidebarOffset;
        editorRatio = Math.max(0.25, Math.min(0.75, relX / availableWidth));
      }
    });
  }

  function handleMouseUp() {
    isDraggingSidebar = false;
    isDraggingSplit = false;
  }
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={handleMouseUp} />

<div
  class="main-layout"
  class:dragging={isDraggingSidebar || isDraggingSplit}
  bind:this={containerEl}
>
  {#if appStore.sidebarVisible}
    <div class="sidebar-pane" style="width: {sidebarWidth}px">
      <Sidebar {oncollectionio} {onruncollection} />
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="drag-handle" onmousedown={startSidebarDrag}></div>
  {/if}

  <div class="editor-pane" style="flex: {editorRatio}">
    <RequestEditor {onimportcurl} />
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="drag-handle" onmousedown={startSplitDrag}></div>

  <div class="response-pane" style="flex: {1 - editorRatio}">
    <ResponseView {ongeneratecode} {oncompare} />
  </div>
</div>

<style>
  .main-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    height: 100%;
  }
  .main-layout.dragging {
    cursor: col-resize;
    user-select: none;
  }
  .sidebar-pane {
    flex-shrink: 0;
    overflow: hidden;
  }
  .editor-pane {
    min-width: 380px;
    overflow: hidden;
  }
  .response-pane {
    min-width: 300px;
    overflow: hidden;
  }
  .drag-handle {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .drag-handle:hover {
    background: var(--color-info);
  }
</style>
