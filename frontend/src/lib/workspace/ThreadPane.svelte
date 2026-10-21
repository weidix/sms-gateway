<script>
  import { tick } from 'svelte';
  import Column from './Column.svelte';
  import MessageRow from './MessageRow.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import { groupMessagesByDay, simDisplayName } from '$lib/format.js';

  let {
    conversation = null,
    messages = [],
    total = 0,
    status = 'idle',
    error = '',
    loadingEarlier = false,
    refreshing = false,
    sims = [],
    onnew,
    onretry,
    onloadearlier,
    onrefresh,
    ondelete,
    onback,
    footer
  } = $props();

  let scrollElement = $state(null);
  let stickToBottom = $state(true);
  let lastSeenId;

  const groups = $derived(groupMessagesByDay(messages));

  const lineLabel = $derived.by(() => {
    const simId = conversation?.sms_preview?.sim_id ?? '';
    if (!simId) return '';
    return simDisplayName(sims.find((sim) => sim.id === simId) ?? null, simId);
  });

  function handleScroll() {
    if (!scrollElement) return;
    const remaining =
      scrollElement.scrollHeight - scrollElement.scrollTop - scrollElement.clientHeight;
    stickToBottom = remaining < 120;
  }

  $effect(() => {
    const currentId = messages.at(-1)?.id;
    const element = scrollElement;
    if (!element || status !== 'ready') {
      lastSeenId = currentId;
      return;
    }
    if (currentId !== lastSeenId && (stickToBottom || lastSeenId === undefined)) {
      element.scrollTo({ top: element.scrollHeight, behavior: 'auto' });
    }
    lastSeenId = currentId;
  });

  async function loadEarlierAndKeepPosition() {
    if (!scrollElement || loadingEarlier) return;
    const previousHeight = scrollElement.scrollHeight;
    const previousTop = scrollElement.scrollTop;
    await onloadearlier?.();
    await tick();
    if (scrollElement) {
      scrollElement.scrollTop = previousTop + (scrollElement.scrollHeight - previousHeight);
    }
  }
</script>

<Column
  title={conversation?.contact?.name || 'Thread'}
  label="Message thread"
  bind:scrollElement
  {footer}
>
  {#snippet meta()}
    {#if conversation && lineLabel}
      <span class="thread__line mono truncate">{lineLabel}</span>
    {/if}
  {/snippet}

  {#snippet actions()}
    {#if conversation}
      <div class="thread__back">
        <IconButton name="arrow-left" label="Back to conversations" onclick={() => onback?.()} />
      </div>
      <IconButton
        name="refresh"
        label="Refresh inbox"
        disabled={refreshing}
        onclick={() => onrefresh?.()}
      />
      <IconButton name="trash" label="Delete conversation" onclick={() => ondelete?.()} />
    {/if}
  {/snippet}

  {#if !conversation}
    <div class="thread-state">
      <EmptyState
        icon="message"
        title="No conversation selected"
        description="Use the + button in the inbox header to start a new message."
      />
    </div>
  {:else if status === 'loading'}
    <div class="thread" aria-busy="true">
      <span class="sr-only">Loading messages</span>
      {#each Array.from({ length: 4 }, (_, index) => index) as index (index)}
        <div class="skeleton" class:skeleton--out={index % 2 === 1} aria-hidden="true">
          <span class="skeleton__bar" style="width: {54 + ((index * 11) % 30)}%"></span>
          <span class="skeleton__bar skeleton__bar--second" style="width: {30 + ((index * 7) % 24)}%"></span>
        </div>
      {/each}
    </div>
  {:else if status === 'error'}
    <div class="thread-state">
      <div class="failure" role="alert">
        <span class="failure__message">{error}</span>
        <Button size="sm" onclick={() => onretry?.()}>Try again</Button>
      </div>
    </div>
  {:else}
    <div class="thread">
      {#if total > messages.length}
        <div class="thread__earlier">
          <Button size="sm" onclick={loadEarlierAndKeepPosition} loading={loadingEarlier}>
            Load earlier
          </Button>
        </div>
      {/if}

      {#each groups as group (group.key)}
        <div class="thread__day mono">{group.label}</div>
        {#each group.items as message (message.id)}
          <MessageRow {message} />
        {/each}
      {/each}

      {#if messages.length === 0}
        <div class="thread-empty">
          <EmptyState icon="message" title="No messages yet" />
        </div>
      {/if}
    </div>
  {/if}
</Column>

<style>
  .thread {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
    min-width: 0;
    padding: 18px 16px 14px;
  }

  .thread__back {
    display: none;
  }

  .thread__line {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    letter-spacing: 0;
    text-transform: none;
    max-width: 180px;
  }

  .thread__earlier {
    display: flex;
    justify-content: center;
    padding-bottom: var(--space-1);
  }

  .thread__day {
    padding: var(--space-1) 0;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    text-align: center;
  }

  .thread-state {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    padding: var(--space-5) var(--space-4);
    width: 100%;
  }

  .thread-empty {
    display: flex;
    flex: 1;
  }

  .skeleton {
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: min(64%, 420px);
    padding: 10px 12px;
    border: 1px solid var(--line-faint);
    border-radius: var(--radius-lg);
    background: var(--surface);
    align-self: flex-start;
    animation: skeleton-pulse 1.6s ease-in-out infinite;
  }

  .skeleton--out {
    align-self: flex-end;
    background: var(--sunken);
  }

  .skeleton__bar {
    display: block;
    height: 8px;
    border-radius: 2px;
    background: color-mix(in srgb, var(--line-strong) 55%, transparent);
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
    width: min(100%, 420px);
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

  @container page (max-width: 899px) {
    .thread__back {
      display: inline-flex;
    }
  }

  @container page (max-width: 560px) {
    .thread__line {
      display: none;
    }
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
