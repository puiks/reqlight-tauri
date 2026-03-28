<script lang="ts">
  let {
    title,
    message,
    icon = "📭",
  }: { title: string; message: string; icon?: string } = $props();
</script>

<div class="empty-state">
  <span class="icon">{icon}</span>
  <h3>{title}</h3>
  <p>{message}</p>
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-sm);
    padding: var(--sp-xl);
    max-width: 280px;
    margin: 0 auto;
    text-align: center;
    height: 100%;
  }
  .icon {
    font-size: var(--fs-icon-lg);
  }
  h3 {
    font-size: var(--fs-callout);
    font-weight: 600;
    color: var(--text-primary);
  }
  p {
    font-size: var(--fs-small);
    color: var(--text-secondary);
    line-height: 1.4;
  }
</style>
