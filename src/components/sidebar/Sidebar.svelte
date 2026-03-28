<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
  import CollectionItem from "./CollectionItem.svelte";
  import HistorySection from "./HistorySection.svelte";
  import SearchBar from "./SearchBar.svelte";
  import EmptyState from "../shared/EmptyState.svelte";
  import ConfirmDialog from "../shared/ConfirmDialog.svelte";
  import ContextMenu from "./ContextMenu.svelte";

  import type { RequestCollection } from "../../lib/types";

  let {
    oncollectionio,
    onruncollection,
  }: {
    oncollectionio: () => void;
    onruncollection?: (collection: RequestCollection) => void;
  } = $props();

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

  function handleRename(id: string, currentName?: string) {
    const name = currentName ?? appStore.collections.find((c) => c.id === id)?.name ?? "";
    renamingId = id;
    renameValue = name;
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

  function handleContextMenuDelete(id: string) {
    const c = appStore.collections.find((col) => col.id === id);
    if (c) handleDelete("collection", c.id, c.name);
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
    <div class="toolbar-actions">
      <button class="tool-btn" onclick={oncollectionio} title="Import / Export">⇄</button>
      <button class="tool-btn add-btn" onclick={handleAddCollection} title="New Collection (⌘⇧N)">+</button>
    </div>
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
              ondelete={() => handleDelete("collection", collection.id, collection.name)}
              ondeleterequest={(id, name) => handleDelete("request", id, name)}
              onrun={() => onruncollection?.(collection)}
            />
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <HistorySection />
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    type={contextMenu.type}
    id={contextMenu.id}
    onaddrequest={handleAddRequest}
    onrename={handleRename}
    ondelete={handleContextMenuDelete}
  />
{/if}

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
  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .tool-btn {
    font-size: var(--fs-callout);
    padding: 0 var(--sp-xs);
    color: var(--text-secondary);
  }
  .tool-btn:hover {
    color: var(--text-primary);
  }
  .add-btn {
    font-size: 18px;
    font-weight: 300;
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
</style>
