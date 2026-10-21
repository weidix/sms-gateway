<script>
  import Column from './Column.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import { conversationTimestamp, formatShortDate } from '$lib/format.js';

  let {
    conversations = [],
    status = 'loading',
    error = '',
    selectedId = null,
    onselect,
    onnew,
    onretry
  } = $props();
</script>

<Column title="Inbox" label="Conversations">
  {#snippet actions()}
    <IconButton name="plus" label="New message" onclick={() => onnew?.()} />
  {/snippet}

  {#if status === 'loading'}
    <div class="list-state" aria-busy="true">
      <span class="sr-only">Loading conversations</span>
      {#each Array.from({ length: 8 }, (_, index) => index) as index (index)}
        <div class="skeleton" aria-hidden="true">
          <span class="skeleton__bar" style="width: {46 + ((index * 13) % 34)}%"></span>
          <span class="skeleton__bar skeleton__bar--second" style="width: {62 + ((index * 7) % 26)}%"></span>
        </div>
      {/each}
    </div>
  {:else if status === 'error'}
    <div class="list-state list-state--center">
      <div class="failure" role="alert">
        <span class="failure__message">{error}</span>
        <Button size="sm" onclick={() => onretry?.()}>Try again</Button>
      </div>
    </div>
  {:else if conversations.length === 0}
    <div class="list-state list-state--center">
      <EmptyState icon="inbox" title="No conversations yet" />
    </div>
  {:else}
    <ul class="list" aria-label="Conversation list">
      {#each conversations as conversation (conversation.contact?.id)}
        {@const contactId = conversation.contact?.id}
        {@const unread = Number(conversation.sms_preview?.status) === 0}
        {@const failed = Number(conversation.sms_preview?.status) === 3}
        <li class="list__item">
          <button
            type="button"
            class="conv"
            class:conv--active={contactId === selectedId}
            aria-current={contactId === selectedId ? 'true' : undefined}
            onclick={() => onselect?.(conversation)}
          >
            <span class="conv__name truncate">{conversation.contact?.name || 'Unknown contact'}</span>
            <span class="conv__time mono">{formatShortDate(conversationTimestamp(conversation))}</span>
            <span class="conv__preview truncate">{conversation.sms_preview?.message || 'No messages'}</span>
            {#if failed}
              <span class="conv__failed">Failed</span>
            {:else if unread}
              <span class="conv__unread" aria-hidden="true"></span>
              <span class="sr-only">Unread</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</Column>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    margin: 0;
    padding: 8px;
    list-style: none;
  }

  .list__item {
    min-width: 0;
  }

  .conv {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-rows: auto auto;
    align-items: center;
    column-gap: 10px;
    row-gap: 2px;
    width: 100%;
    padding: 10px 12px;
    border: 0;
    border-radius: 10px;
    background: transparent;
    color: inherit;
    text-align: left;
    transition: background var(--dur) ease;
  }


  .conv:hover {
    background: var(--sunken);
  }

  .conv:focus-visible {
    outline-offset: -2px;
  }

  .conv--active {
    background: var(--sunken);
  }

  .conv__name {
    grid-column: 1;
    grid-row: 1;
    color: var(--ink);
    font-size: 14px;
    font-weight: 600;
    line-height: 1.3;
  }

  .conv__time {
    grid-column: 2;
    grid-row: 1;
    justify-self: end;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .conv__preview {
    grid-column: 1;
    grid-row: 2;
    color: var(--ink-3);
    font-size: 12.5px;
    line-height: 1.35;
  }

  .conv__failed {
    grid-column: 2;
    grid-row: 2;
    justify-self: end;
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
  }

  .conv__unread {
    grid-column: 2;
    grid-row: 2;
    justify-self: end;
    width: 6px;
    height: 6px;
    margin-right: 2px;
    border-radius: 50%;
    background: var(--ink);
  }

  .list-state {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .list-state--center {
    flex: 1;
    align-items: center;
    justify-content: center;
    padding: var(--space-4) var(--space-3);
  }

  .skeleton {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 11px 12px;
    border-bottom: 1px solid var(--line-faint);
  }

  .skeleton__bar {
    display: block;
    height: 8px;
    border-radius: 2px;
    background: var(--sunken);
    animation: skeleton-pulse 1.6s ease-in-out infinite;
  }

  .skeleton__bar--second {
    height: 7px;
    opacity: 0.8;
  }

  .failure {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .failure__message {
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-xs);
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  @keyframes skeleton-pulse {
    0%,
    100% {
      opacity: 0.55;
    }
    50% {
      opacity: 1;
    }
  }
</style>
