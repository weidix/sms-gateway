<script>
  /**
   * Column frame shared by the three workspace regions. The header and footer
   * sit inside the scroll container so their inner edges line up with the
   * scrolled rows (the reserved scrollbar gutter), while both stay pinned.
   */
  let {
    title = '',
    label = '',
    meta,
    actions,
    footer,
    scrollElement = $bindable(null),
    children
  } = $props();
</script>

<section class="column" aria-label={label || title}>
  <div class="column__scroll scroll-y" bind:this={scrollElement}>
    <header class="column__head">
      <h2 class="column__title truncate">{title}</h2>
      {#if meta}<div class="column__meta">{@render meta()}</div>{/if}
      {#if actions}<div class="column__actions">{@render actions()}</div>{/if}
    </header>

    <div class="column__content">{@render children?.()}</div>

    {#if footer}
      <div class="column__foot">{@render footer()}</div>
    {/if}
  </div>
</section>

<style>
  .column {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    border: 0;
    border-radius: var(--radius);
    background: var(--raised);
    box-shadow: var(--shadow-card);
    overflow: hidden;
  }

  .column__scroll {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .column__head {
    position: sticky;
    top: 0;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    height: 54px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
    background: var(--raised);
  }

  .column__title {
    flex: 1 1 auto;
    min-width: 0;
    margin: 0;
    padding-block: 1px;
    font-family: var(--font-sans);
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
    /* Glyph descenders at some zoom levels clipped by 1–2px when the line box
       was 1.2; 1.5 plus a hair of padding keeps the text fully inside the
       overflow-hidden ellipsis box. */
    line-height: 1.5;
    color: var(--ink);
  }

  .column__meta {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-3);
    white-space: nowrap;
  }

  .column__actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  .column__content {
    display: flex;
    flex-direction: column;
    flex: 1 0 auto;
    min-width: 0;
  }

  .column__foot {
    position: sticky;
    bottom: 0;
    z-index: 3;
    flex: none;
    border-top: 1px solid var(--line);
    background: var(--raised);
  }
</style>
