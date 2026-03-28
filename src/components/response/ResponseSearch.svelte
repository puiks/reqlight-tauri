<script lang="ts">
  import type { SearchMatch } from "../../lib/utils/text-search";

  let {
    searchQuery = $bindable(),
    searchResults,
    activeMatchIndex = $bindable(),
    searchInputEl = $bindable(),
    onclose,
  }: {
    searchQuery: string;
    searchResults: SearchMatch[];
    activeMatchIndex: number;
    searchInputEl: HTMLInputElement | null;
    onclose: () => void;
  } = $props();

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    } else if (e.key === "Enter") {
      if (e.shiftKey) { prevMatch(); } else { nextMatch(); }
    }
  }

  function nextMatch() {
    if (searchResults.length === 0) return;
    activeMatchIndex = (activeMatchIndex + 1) % searchResults.length;
    scrollToActiveMatch();
  }

  function prevMatch() {
    if (searchResults.length === 0) return;
    activeMatchIndex = (activeMatchIndex - 1 + searchResults.length) % searchResults.length;
    scrollToActiveMatch();
  }

  function scrollToActiveMatch() {
    requestAnimationFrame(() => {
      const el = document.querySelector(".search-match.active");
      el?.scrollIntoView({ block: "center", behavior: "smooth" });
    });
  }
</script>

<div class="search-bar">
  <input
    type="text"
    class="search-input"
    placeholder="Search in response..."
    bind:value={searchQuery}
    bind:this={searchInputEl}
    onkeydown={handleSearchKeydown}
    spellcheck="false"
  />
  {#if searchQuery}
    <span class="search-count">
      {searchResults.length > 0
        ? `${activeMatchIndex + 1}/${searchResults.length}`
        : "0 results"}
    </span>
    <button class="nav-btn" onclick={prevMatch} title="Previous (Shift+Enter)">▲</button>
    <button class="nav-btn" onclick={nextMatch} title="Next (Enter)">▼</button>
  {/if}
</div>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: var(--sp-xs) var(--sp-md);
    border-bottom: 1px solid var(--border-light);
    background: var(--bg-secondary);
  }
  .search-input {
    flex: 1;
    font-size: var(--fs-small);
    padding: 2px var(--sp-sm);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-input);
  }
  .search-count {
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .nav-btn {
    font-size: var(--fs-caption);
    padding: 0 2px;
    color: var(--text-secondary);
  }
</style>
