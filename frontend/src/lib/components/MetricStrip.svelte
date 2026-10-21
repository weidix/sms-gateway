<script>
  import Metric from './Metric.svelte';

  let { items = [], cols = 4 } = $props();
</script>

<div class="metric-strip" style="--cols: {Math.min(cols, Math.max(items.length, 1))}">
  {#each items as item (item.label)}
    <Metric label={item.label} value={item.value ?? '—'} unit={item.unit ?? ''} hint={item.hint ?? ''} />
  {/each}
</div>
