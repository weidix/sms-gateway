<script>
  import Icon from './Icon.svelte';

  let { icon = 'inbox', title = 'Nothing here yet', description = '', children } = $props();
</script>

<div class="empty">
  <span class="empty__mark"><Icon name={icon} size={20} /></span>
  <h3 class="empty__title">{title}</h3>
  {#if description}<p class="empty__desc">{description}</p>{/if}
  {#if children}<div class="empty__action">{@render children()}</div>{/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: var(--space-2);
    padding: var(--space-7) var(--space-5);
    color: var(--ink-3);
  }

  .empty__mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    margin-bottom: var(--space-1);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--raised);
    box-shadow: var(--shadow-card);
    color: var(--ink-3);
  }

  .empty__title {
    font-family: var(--font-sans);
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--ink);
    letter-spacing: -0.01em;
  }

  .empty__desc {
    font-size: var(--text-sm);
    line-height: 1.5;
    max-width: 40ch;
    color: var(--ink-3);
  }

  .empty__action {
    margin-top: var(--space-2);
  }
</style>
