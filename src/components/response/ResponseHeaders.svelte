<script lang="ts">
  import type { ResponseRecord } from "../../lib/types";

  let { response }: { response: ResponseRecord } = $props();
</script>

<div class="response-headers">
  <table>
    <thead>
      <tr>
        <th>Name</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      {#each response.headers as header}
        <tr>
          <td class="header-name">{header.key}</td>
          <td class="header-value">{header.value}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .response-headers {
    padding: var(--sp-sm);
    overflow: auto;
    height: 100%;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--fs-small);
  }
  th {
    text-align: left;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-sm);
    border-bottom: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: var(--fs-footnote);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  td {
    padding: var(--sp-xs) var(--sp-sm);
    border-bottom: 1px solid var(--border-light);
    user-select: text;
  }
  .header-name {
    font-weight: 500;
    color: var(--text-primary);
    width: 200px;
    font-family: var(--font-mono);
  }
  .header-value {
    color: var(--text-secondary);
    font-family: var(--font-mono);
    word-break: break-all;
  }
</style>
