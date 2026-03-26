<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
</script>

<div class="search-bar">
  <input
    type="text"
    placeholder="Search requests..."
    bind:value={appStore.searchQuery}
  />
</div>

<style>
  .search-bar {
    padding: var(--sp-sm);
  }
  input {
    width: 100%;
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-sm);
  }
</style>
