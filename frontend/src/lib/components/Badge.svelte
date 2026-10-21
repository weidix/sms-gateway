<script>
  let { tone = 'neutral', dot = false, children } = $props();
</script>

<span class="badge badge--{tone}">
  {#if dot}<span class="badge__dot" aria-hidden="true"></span>{/if}
  {@render children?.()}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 22px;
    padding: 0 9px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--raised);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    font-weight: 500;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    line-height: 1;
    white-space: nowrap;
  }

  .badge__dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: currentColor;
  }

  .badge--outline {
    background: transparent;
    border-color: var(--line-strong);
    color: var(--ink-3);
  }

  .badge--ok {
    border-color: color-mix(in srgb, var(--ok) 32%, var(--line));
    color: var(--ok);
    background: color-mix(in srgb, var(--ok) 6%, var(--raised));
  }

  .badge--warn {
    border-color: color-mix(in srgb, var(--warn) 32%, var(--line));
    color: var(--warn);
    background: color-mix(in srgb, var(--warn) 6%, var(--raised));
  }

  .badge--danger {
    border-color: color-mix(in srgb, var(--danger) 32%, var(--line));
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 6%, var(--raised));
  }

  .badge--idle {
    color: var(--idle);
  }
</style>
