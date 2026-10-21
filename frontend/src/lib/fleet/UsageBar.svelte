<script>
  import { ratioPercent } from '$lib/format.js';

  let { label = 'Read', used = 0, total = 0, storage = '' } = $props();

  const percent = $derived(ratioPercent(used, total));
</script>

<div class="usage">
  <span class="usage__label">{label}</span>
  <span
    class="usage__track"
    role="progressbar"
    aria-label={`${label} storage usage`}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={percent}
  >
    <span class="usage__fill" style="width: {percent}%"></span>
  </span>
  <span class="usage__count mono">{used}/{total}</span>
  {#if storage}
    <span class="usage__storage mono">{storage}</span>
  {/if}
</div>

<style>
  .usage {
    display: grid;
    grid-template-columns: 54px minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .usage__label {
    font-size: var(--text-xs);
    color: var(--ink-3);
  }

  .usage__track {
    display: block;
    height: 6px;
    border-radius: 3px;
    background: var(--sunken);
    overflow: hidden;
  }

  .usage__fill {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--ink-2);
  }

  .usage__count {
    font-size: var(--text-xs);
    color: var(--ink-2);
    white-space: nowrap;
  }

  .usage__storage {
    font-size: var(--text-2xs);
    color: var(--ink-3);
    letter-spacing: 0.06em;
  }
</style>
