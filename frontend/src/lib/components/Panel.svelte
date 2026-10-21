<script>
  let {
    eyebrow = '',
    title = '',
    description = '',
    flush = false,
    actions,
    children
  } = $props();
</script>

<section class="panel">
  {#if eyebrow || title || actions}
    <header class="panel__head">
      <div class="panel__heading">
        {#if eyebrow}<span class="eyebrow">{eyebrow}</span>{/if}
        {#if title}<h2 class="panel__title">{title}</h2>{/if}
        {#if description}<p class="panel__desc">{description}</p>{/if}
      </div>
      {#if actions}
        <div class="panel__actions">{@render actions()}</div>
      {/if}
    </header>
  {/if}
  <div class="panel__body" class:panel__body--flush={flush}>
    {@render children?.()}
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 0;
    border-radius: var(--radius);
    background: var(--raised);
    box-shadow: var(--shadow-card);
  }

  .panel__head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: 15px 18px 13px;
    border-bottom: 1px solid var(--line);
    min-height: 52px;
  }

  .panel__heading {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .panel__title {
    font-family: var(--font-sans);
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
    line-height: 1.25;
  }

  .panel__desc {
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.45;
    max-width: 62ch;
  }

  .panel__actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: none;
  }

  .panel__body {
    padding: 18px;
    min-width: 0;
  }

  .panel__body--flush {
    padding: 0;
  }
</style>
