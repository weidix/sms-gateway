<script>
  /** SIM card + signal mark. Replaces the old two-letter badge. */
  let { size = 28, showWordmark = true, label = 'SMS Gateway' } = $props();
</script>

<span class="logo" aria-label={label}>
  <svg width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
    <rect class="logo__tile" x="0" y="0" width="32" height="32" rx="9" />
    <path
      class="logo__card"
      d="M11 6.5h7.5L23 11.5v11a2.5 2.5 0 0 1-2.5 2.5h-9A2.5 2.5 0 0 1 9 22.5V9a2.5 2.5 0 0 1 2-2.5z"
    />
    <path class="logo__bars" d="M12.5 20.5v-3M16 20.5v-5.2M19.5 20.5v-7.4" />
  </svg>
  {#if showWordmark}<span class="logo__word">SMS Gateway</span>{/if}
</span>

<style>
  .logo {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }

  .logo__tile {
    fill: var(--ink);
  }

  .logo__card {
    fill: none;
    stroke: var(--paper);
    stroke-width: 1.7;
    stroke-linejoin: round;
  }

  .logo__bars {
    fill: none;
    stroke: var(--paper);
    stroke-width: 1.9;
    stroke-linecap: round;
  }

  .logo__word {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--ink);
    white-space: nowrap;
  }

  @media (max-width: 640px) {
    .logo__word {
      display: none;
    }
  }
</style>
