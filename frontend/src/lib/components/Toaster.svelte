<script>
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { toast } from '../stores/toast.svelte.js';
  import Icon from './Icon.svelte';

  const toneIcon = { ok: 'check', warn: 'alert', danger: 'alert', neutral: 'info' };
</script>

<section class="toaster" aria-live="polite" aria-label="Notifications">
  {#each toast.items as entry (entry.id)}
    <div
      class="toast toast--{entry.tone}"
      role="status"
      in:fly={{ y: 8, duration: 190, easing: cubicOut }}
      out:fade={{ duration: 120 }}
    >
      <span class="toast__icon"><Icon name={toneIcon[entry.tone]} size={14} /></span>
      <div class="toast__text">
        {#if entry.title}<span class="toast__title">{entry.title}</span>{/if}
        <span class="toast__message">{entry.message}</span>
      </div>
      <button
        type="button"
        class="toast__close"
        aria-label="Dismiss notification"
        onclick={() => toast.dismiss(entry.id)}
      >
        <Icon name="close" size={13} />
      </button>
    </div>
  {/each}
</section>

<style>
  .toaster {
    position: fixed;
    right: 20px;
    bottom: 128px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    pointer-events: none;
    max-width: min(380px, calc(100vw - 32px));
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 10px 10px 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--raised);
    color: var(--ink);
    box-shadow: var(--shadow-pop);
  }

  .toast__icon {
    display: inline-flex;
    margin-top: 1px;
    color: var(--ink-3);
    flex: none;
  }

  .toast--ok .toast__icon {
    color: var(--ok);
  }

  .toast--warn .toast__icon {
    color: var(--warn);
  }

  .toast--danger .toast__icon {
    color: var(--danger);
  }

  .toast__text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .toast__title {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-3);
  }

  .toast__message {
    font-size: var(--text-sm);
    line-height: 1.45;
    word-break: break-word;
  }

  .toast__close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    flex: none;
  }

  .toast__close:hover {
    background: var(--sunken);
    color: var(--ink);
  }

  @media (max-width: 760px) {
    .toaster {
      right: 16px;
      bottom: 176px;
    }
  }
</style>
