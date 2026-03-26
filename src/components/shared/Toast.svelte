<script lang="ts">
  import { appStore } from "../../lib/stores/app.svelte";
</script>

{#if appStore.toastMessage}
  <div class="toast" role="alert">
    {appStore.toastMessage}
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--text-primary);
    color: var(--text-inverse);
    padding: var(--sp-sm) var(--sp-lg);
    border-radius: var(--radius-lg);
    font-size: var(--fs-small);
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    animation: fadeIn 0.15s ease-out;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }
</style>
