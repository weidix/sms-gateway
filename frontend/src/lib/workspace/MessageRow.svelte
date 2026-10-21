<script>
  import { formatTime } from '$lib/format.js';
  import MessageBubble from './MessageBubble.svelte';

  let { message = null } = $props();

  const outbound = $derived(Boolean(message?.send));
  const status = $derived(Number(message?.status ?? 0));
</script>

<article class="msg" class:msg--out={outbound}>
  <MessageBubble {outbound}>
    <p class="msg__text">{message?.message ?? ''}</p>
    {#if outbound && status === 2}
      <span class="msg__status">Sending…</span>
    {:else if outbound && status === 3}
      <span class="msg__status msg__status--failed">Not sent</span>
    {/if}
  </MessageBubble>
  <span class="msg__time mono">{formatTime(message?.timestamp)}</span>
</article>

<style>
  .msg {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    max-width: 100%;
  }

  .msg--out {
    align-items: flex-end;
  }

  .msg__text {
    margin: 0;
    color: var(--ink);
    font-size: 13.5px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .msg__status {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-3);
  }

  .msg__status--failed {
    color: var(--danger);
  }

  .msg__time {
    padding: 0 2px;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
</style>
