<script lang="ts">
  import { METHOD_COLORS, type HttpMethod } from "../../lib/types";

  let { method }: { method: HttpMethod } = $props();
</script>

<span class="badge" style="color: {METHOD_COLORS[method]}">
  {method}
</span>

<style>
  .badge {
    font-size: var(--fs-footnote);
    font-weight: 600;
    font-family: var(--font-mono);
    width: 38px;
    display: inline-block;
    text-align: center;
    flex-shrink: 0;
  }
</style>
