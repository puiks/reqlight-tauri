<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import CollectionItem from "./CollectionItem.svelte";
  import HistorySection from "./HistorySection.svelte";
  import SearchBar from "./SearchBar.svelte";
  import EmptyState from "../shared/EmptyState.svelte";
  import ConfirmDialog from "../shared/ConfirmDialog.svelte";

  let contextMenu = $state<{
    x: number;
    y: number;
    type: "collection" | "request";
    id: string;
    collectionId?: string;
  } | null>(null);

  let confirmDelete = $state<{
    type: "collection" | "request";
    id: string;
    name: string;
  } | null>(null);

  let renamingId = $state<string | null>(null);
  let renameValue = $state("");
  let dragFromIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function handleCollectionContext(e: MouseEvent, collectionId: string) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, type: "collection", id: collectionId };
  }

  function handleAddCollection() {
    appStore.addCollection();
  }

  function handleAddRequest(collectionId: string) {
    appStore.addRequest(collectionId);
    contextMenu = null;
  }

  function handleRename(id: string, currentName: string) {
    renamingId = id;
    renameValue = currentName;
    contextMenu = null;
  }

  function commitRename() {
    if (renamingId && renameValue.trim()) {
      appStore.renameCollection(renamingId, renameValue.trim());
    }
    renamingId = null;
  }

  function handleDelete(type: "collection" | "request", id: string, name: string) {
    confirmDelete = { type, id, name };
    contextMenu = null;
  }

  function confirmDeleteAction() {
    if (!confirmDelete) return;
    if (confirmDelete.type === "collection") {
      appStore.deleteCollection(confirmDelete.id);
    } else {
      appStore.deleteRequest(confirmDelete.id);
    }
    confirmDelete = null;
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleCollectionDragStart(index: number, e: DragEvent) {
    dragFromIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", `collection:${index}`);
    }
  }

  function handleCollectionDragOver(index: number, e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverIndex = index;
  }

  function handleCollectionDrop(index: number, e: DragEvent) {
    e.preventDefault();
    if (dragFromIndex !== null && dragFromIndex !== index) {
      appStore.reorderCollection(dragFromIndex, index);
    }
    dragFromIndex = null;
    dragOverIndex = null;
  }

  function handleCollectionDragEnd() {
    dragFromIndex = null;
    dragOverIndex = null;
  }
</script>

<svelte:window onclick={closeContextMenu} />

<div class="sidebar">
  <div class="toolbar">
    <span class="title">Collections</span>
    <button class="add-btn" onclick={handleAddCollection} title="New Collection (⌘⇧N)">+</button>
  </div>

  <SearchBar />

  <div class="list">
    {#if appStore.filteredCollections.length === 0}
      {#if appStore.collections.length === 0}
        <EmptyState
          icon="📁"
          title="No Collections"
          message="Create a collection to organize your requests."
        />
      {:else}
        <EmptyState
          icon="🔍"
          title="No Results"
          message="No requests match your search."
        />
      {/if}
    {:else}
      {#each appStore.filteredCollections as collection, index (collection.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="collection-drag-wrapper"
          class:collection-drag-over={dragOverIndex === index && dragFromIndex !== index}
          class:collection-dragging={dragFromIndex === index}
          draggable="true"
          ondragstart={(e) => handleCollectionDragStart(index, e)}
          ondragover={(e) => handleCollectionDragOver(index, e)}
          ondrop={(e) => handleCollectionDrop(index, e)}
          ondragend={handleCollectionDragEnd}
          ondragleave={() => { if (dragOverIndex === index) dragOverIndex = null; }}
        >
          {#if renamingId === collection.id}
            <div class="rename-input">
              <input
                type="text"
                bind:value={renameValue}
                onblur={commitRename}
                onkeydown={(e) => {
                  if (e.key === "Enter") commitRename();
                  if (e.key === "Escape") (renamingId = null);
                }}
              />
            </div>
          {:else}
            <CollectionItem
              {collection}
              oncontextmenu={(e) => handleCollectionContext(e, collection.id)}
            />
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <HistorySection />
</div>

<!-- Context Menu -->
{#if contextMenu}
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    role="menu"
  >
    {#if contextMenu.type === "collection"}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="menu-item" role="menuitem" tabindex="-1" onclick={() => handleAddRequest(contextMenu!.id)}>
        New Request
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="menu-item" role="menuitem" tabindex="-1"
        onclick={() => {
          const c = appStore.collections.find((c) => c.id === contextMenu!.id);
          if (c) handleRename(c.id, c.name);
        }}
      >
        Rename
      </div>
      <div class="menu-divider"></div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="menu-item danger" role="menuitem" tabindex="-1"
        onclick={() => {
          const c = appStore.collections.find((c) => c.id === contextMenu!.id);
          if (c) handleDelete("collection", c.id, c.name);
        }}
      >
        Delete
      </div>
    {/if}
  </div>
{/if}

<!-- Confirm Dialog -->
{#if confirmDelete}
  <ConfirmDialog
    title={`Delete ${confirmDelete.type === 'collection' ? 'Collection' : 'Request'}`}
    message={`Are you sure you want to delete "${confirmDelete.name}"? This action cannot be undone.`}
    onconfirm={confirmDeleteAction}
    oncancel={() => (confirmDelete = null)}
  />
{/if}

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    overflow: hidden;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-sm) var(--sp-md);
    border-bottom: 1px solid var(--border-color);
  }
  .title {
    font-size: var(--fs-small);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }
  .add-btn {
    font-size: 18px;
    font-weight: 300;
    padding: 0 var(--sp-xs);
    color: var(--text-secondary);
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-xs);
  }
  .rename-input {
    padding: var(--sp-xs) var(--sp-sm);
  }
  .rename-input input {
    width: 100%;
    font-size: var(--fs-small);
  }

  /* Collection drag */
  .collection-drag-wrapper {
    border-top: 2px solid transparent;
    transition: border-color 0.1s;
  }
  .collection-drag-wrapper.collection-drag-over {
    border-top-color: var(--color-info);
  }
  .collection-drag-wrapper.collection-dragging {
    opacity: 0.5;
  }

  /* Context menu */
  .context-menu {
    position: fixed;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    padding: var(--sp-xs) 0;
    min-width: 160px;
    z-index: 50;
  }
  .menu-item {
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-small);
    cursor: pointer;
  }
  .menu-item:hover {
    background: var(--bg-hover);
  }
  .menu-item.danger {
    color: var(--color-error);
  }
  .menu-divider {
    height: 1px;
    background: var(--border-color);
    margin: var(--sp-xs) 0;
  }
</style>
