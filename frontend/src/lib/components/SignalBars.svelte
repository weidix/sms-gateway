<script>
  let { rssi = null, size = 'md', showValue = false } = $props();

  const LEVELS = [4, 9, 14, 19];
  const known = $derived(Number.isFinite(rssi) && rssi >= 0 && rssi <= 31);
  const level = $derived(known ? LEVELS.filter((threshold) => rssi > threshold).length + 1 : 0);
  const percent = $derived(known ? Math.round((rssi / 31) * 100) : null);
</script>

<span
  class="signal signal--{size}"
  role="img"
  aria-label={known ? `Signal quality ${rssi} of 31` : 'Signal quality unknown'}
>
  <span class="signal__bars" aria-hidden="true">
    {#each [1, 2, 3, 4] as bar}
      <span class="signal__bar" class:signal__bar--on={bar <= level} style="--bar: {bar}"></span>
    {/each}
  </span>
  {#if showValue}
    <span class="signal__value mono">{known ? `${percent}%` : '—'}</span>
  {/if}
</span>

<style>
  .signal {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    line-height: 1;
  }

  .signal__bars {
    display: inline-flex;
    align-items: flex-end;
    gap: 2px;
    height: 12px;
  }

  .signal__bar {
    width: 3px;
    height: calc(3px + (var(--bar) - 1) * 3px);
    background: var(--line-strong);
    border-radius: 0.5px;
  }

  .signal__bar--on {
    background: var(--ink);
  }

  .signal__value {
    font-size: var(--text-xs);
    color: var(--ink-3);
  }

  .signal--lg .signal__bars {
    height: 16px;
    gap: 2.5px;
  }

  .signal--lg .signal__bar {
    width: 4px;
    height: calc(4px + (var(--bar) - 1) * 4px);
  }
</style>
