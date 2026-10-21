<script>
  import { Dialog as DialogPrimitive } from 'bits-ui';
  import Icon from './Icon.svelte';

  let {
    open = $bindable(false),
    title = '',
    description = '',
    size = 'md',
    children,
    footer
  } = $props();
</script>

<DialogPrimitive.Root bind:open>
  <DialogPrimitive.Portal>
    <DialogPrimitive.Overlay class="dialog-overlay" />
    <DialogPrimitive.Content class="dialog dialog--{size}">
      <header class="dialog__head">
        <div class="dialog__heading">
          <DialogPrimitive.Title class="dialog__title">{title}</DialogPrimitive.Title>
          {#if description}
            <DialogPrimitive.Description class="dialog__desc">{description}</DialogPrimitive.Description>
          {/if}
        </div>
        <button
          type="button"
          class="dialog__close"
          aria-label="Close dialog"
          onclick={() => (open = false)}
        >
          <Icon name="close" size={15} />
        </button>
      </header>

      <div class="dialog__body">{@render children?.()}</div>

      {#if footer}
        <footer class="dialog__footer">{@render footer()}</footer>
      {/if}
    </DialogPrimitive.Content>
  </DialogPrimitive.Portal>
</DialogPrimitive.Root>

<style>
  :global(.dialog-overlay) {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: rgba(18, 22, 19, 0.38);
    animation: fade-in 120ms ease-out;
  }

  :global(.dialog) {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 61;
    display: flex;
    flex-direction: column;
    width: calc(100vw - 32px);
    max-height: min(86vh, 760px);
    transform: translate(-50%, -50%);
    border: 0;
    border-radius: var(--radius-lg);
    background: var(--raised);
    box-shadow: var(--shadow-pop);
    animation: dialog-in 140ms cubic-bezier(0.2, 0.8, 0.3, 1);
  }

  :global(.dialog--sm) {
    max-width: 400px;
  }

  :global(.dialog--md) {
    max-width: 500px;
  }

  :global(.dialog--lg) {
    max-width: 640px;
  }

  :global(.dialog--xl) {
    max-width: 860px;
  }

  :global(.dialog__head) {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: 20px 22px 16px;
    border-bottom: 1px solid var(--line);
  }

  :global(.dialog__heading) {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }

  :global(.dialog__title) {
    font-family: var(--font-sans);
    font-size: var(--text-xl);
    font-weight: 600;
    line-height: 1.15;
    letter-spacing: -0.015em;
    color: var(--ink);
  }

  :global(.dialog__desc) {
    margin: 0;
    font-size: var(--text-sm);
    line-height: 1.5;
    color: var(--ink-3);
    max-width: 58ch;
  }

  :global(.dialog__close) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    margin: -4px -6px 0 0;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    flex: none;
  }

  :global(.dialog__close:hover) {
    background: var(--sunken);
    color: var(--ink);
  }

  :global(.dialog__body) {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
    padding: 18px 22px;
  }

  :global(.dialog__footer) {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: 14px 22px;
    border-top: 1px solid var(--line);
    background: var(--surface);
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  @keyframes dialog-in {
    from {
      opacity: 0;
      transform: translate(-50%, -48%) scale(0.985);
    }
  }
</style>
