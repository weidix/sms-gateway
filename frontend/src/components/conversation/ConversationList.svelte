<script>
  import Icon from "@iconify/svelte";
  import { formatDate } from "../../js/dateFormat";
  import { flip } from "svelte/animate";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    conversations,
    currentContact,
    changeCurrentConversation,
    conversationLoading,
    deleteConversation,
  } from "../../stores/conversation";
  import { generateUUID } from "../../js/uuid";
  import { simCards } from "../../stores/simcards";

  let { onConversationSelect = () => {} } = $props();

  const SmsStatus = {
    Unread: 0,
    Read: 1,
    Loading: 2,
    Failed: 3,
  };

  let searchTemporaryIsActive = $state(false);
  let searchTemporaryValue = $state("");

  // Function to get SIM card alias by ID
  function getSimCardDisplayName(simId) {
    if (!simId) {
      return "Unknown";
    }

    const simCard = $simCards.find(sim => sim.id === simId);
    if (simCard) {
      return simCard.alias || simCard.phone_number || `SIM ${simId.slice(-8)}`;
    }
    return `SIM ${simId.slice(-8)}`;
  }

  let filteredConversations = $derived(
    $conversationLoading
      ? []
      : $conversations.filter(
          (conversation) =>
            searchTemporaryValue === "" ||
            conversation.contact.name
              .toLowerCase()
              .includes(searchTemporaryValue.toLowerCase())
        )
  );

  function searchHandleFocus() {
    searchTemporaryIsActive = true;
  }

  function searchHandleBlur() {
    searchTemporaryIsActive = false;
  }

  function conversationHandleClick(conversation) {
    changeCurrentConversation(conversation.contact);
    onConversationSelect();
  }

  function createNewMessage() {
    // Check if there's already a new message item
    const existingNewMessage = $conversations.find(conv => conv.contact.new === true);

    if (existingNewMessage) {
      if ($currentContact?.id !== existingNewMessage.contact.id) {
        changeCurrentConversation(existingNewMessage.contact);
      } else {
        window.dispatchEvent(
          new CustomEvent("open-new-recipient-dialog", {
            detail: { reset: true },
          }),
        );
      }
      onConversationSelect();
      return;
    }

    // Create new message item
    const uuid = generateUUID();
    changeCurrentConversation({
      id: uuid,
      name: "新信息",
      new: true,
    });
    onConversationSelect();
  }

  function deleteConversationHandleClick(conversation) {
    if ($conversations.length === 1) {
      return;
    }
    deleteConversation(conversation.contact.id);
    changeCurrentConversation($conversations[0]?.contact);
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="flex items-center gap-2 px-3 pb-2 pt-3">
    <div class="relative min-w-0 flex-1">
      <Icon
        icon="carbon:search"
        class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2"
        style="color: var(--text-muted);"
      />
      <input
        type="text"
        onfocus={searchHandleFocus}
        onblur={searchHandleBlur}
        bind:value={searchTemporaryValue}
        class={`shell-input h-8 pl-9 pr-8 ${searchTemporaryIsActive ? 'border-[color:var(--line-strong)]' : ''}`}
        placeholder="Search conversations..."
      />
      {#if searchTemporaryValue}
        <button
          onclick={() => searchTemporaryValue = ""}
          class="absolute right-1.5 top-1/2 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
          aria-label="Clear search"
        >
          <Icon icon="carbon:close" class="h-3.5 w-3.5" style="color: var(--text-muted);" />
        </button>
      {/if}
    </div>

    <button
      class="shell-button shell-button-primary h-8 w-8 shrink-0 px-0"
      onclick={createNewMessage}
      title="New Message"
      aria-label="New Message"
    >
      <Icon icon="carbon:add" class="h-4 w-4" />
    </button>
  </div>

  <div class="min-h-0 flex-1 overflow-hidden">
    <div class="conversation-scroll-region shell-scrollbar h-full overflow-y-auto pr-1">
      <div class="space-y-px px-3 pb-2">
        {#if $conversationLoading}
          {#each Array(5) as _}
            <div class="flex items-center gap-2.5 rounded-md p-2.5">
              <div class="h-9 w-9 shrink-0 animate-pulse rounded-md bg-black/10 dark:bg-white/10"></div>
              <div class="flex-1 space-y-2">
                <div class="h-3 w-32 animate-pulse rounded-full bg-black/10 dark:bg-white/10"></div>
                <div class="h-3 w-24 animate-pulse rounded-full bg-black/10 dark:bg-white/10"></div>
              </div>
            </div>
          {/each}
        {:else}
          {#each filteredConversations as conversation, index (conversation.contact.id)}
            {@const isCurrent = $currentContact?.id === conversation.contact.id}
            {@const hasPreviewContent = Boolean(
              conversation.sms_preview?.message || conversation.sms_preview?.sim_id
            )}
            <div
              animate:flip={{ duration: 200, easing: cubicOut }}
              transition:fade={{ duration: 150 }}
              class="relative focus:outline-none"
              role="button"
              tabindex="0"
              onclick={() => conversationHandleClick(conversation)}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  conversationHandleClick(conversation);
                }
              }}
            >
              <div
                class={`group relative flex items-center gap-2.5 rounded-md px-2 py-2 transition-colors duration-100 ${
                  isCurrent
                    ? 'bg-[var(--panel-soft)]'
                    : 'hover:bg-[var(--accent-soft)]'
                }`}
              >
                <div class="relative flex-shrink-0">
                  <div class={`flex h-9 w-9 items-center justify-center rounded-md border ${
                    isCurrent
                      ? 'border-transparent bg-[var(--ink-strong)] text-[var(--paper-strong)]'
                      : 'border-[color:var(--line-soft)] bg-[var(--panel-strong)] text-[var(--text-secondary)]'
                  }`}>
                    <Icon icon="carbon:user-avatar" class="h-4 w-4" />
                  </div>

                  {#if conversation.sms_preview?.status === SmsStatus.Unread}
                    <div
                      class="absolute -right-0.5 -top-0.5 h-2 w-2 rounded-full ring-2"
                      style="background: var(--text-strong); --tw-ring-color: var(--panel);"
                      transition:scale={{ duration: 150 }}
                    ></div>
                  {/if}
                </div>

                <div class="min-w-0 flex-1">
                  <div class="mb-0.5 flex items-start justify-between gap-2">
                    <h3 class="truncate pr-2 text-sm font-medium" style="color: var(--text-strong);">
                      {conversation.contact.name}
                    </h3>

                    {#if !conversation.contact.new && conversation.sms_preview?.timestamp}
                      <span
                        class="shell-mono shrink-0 text-[10px]"
                        style="color: var(--text-muted);"
                      >
                        {formatDate(conversation.sms_preview.timestamp)}
                      </span>
                    {/if}
                  </div>

                  <div class="flex items-center gap-1.5">
                    {#if conversation.sms_preview && hasPreviewContent}
                      <span
                        class="shell-mono shrink-0 rounded-sm border px-1 py-px text-[10px] leading-4"
                        style="border-color: var(--line-soft); color: var(--text-muted);"
                      >
                        {getSimCardDisplayName(conversation.sms_preview.sim_id)}
                      </span>
                      <p
                        class="line-clamp-1 flex-1 truncate text-xs leading-5"
                        style="color: var(--text-secondary);"
                      >
                        {conversation.sms_preview.message}
                      </p>
                    {:else}
                      <span class="text-xs italic" style="color: var(--text-muted);">
                        {conversation.contact.new ? 'Recipient draft' : 'No messages yet'}
                      </span>
                    {/if}
                  </div>
                </div>

                {#if conversation.contact.new === true}
                  <button
                    class="flex h-7 w-7 shrink-0 items-center justify-center rounded opacity-0 transition-all duration-150 group-hover:opacity-100 hover:bg-black/5 dark:hover:bg-white/5"
                    onclick={(e) => {
                      e.stopPropagation();
                      deleteConversationHandleClick(conversation);
                    }}
                    title="Delete"
                  >
                    <Icon icon="carbon:trash-can" class="h-3.5 w-3.5" style="color: var(--text-muted);" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        {/if}

        {#if !$conversationLoading && filteredConversations.length === 0}
          <div class="flex flex-col items-center justify-center px-6 py-12 text-center">
            <div class="shell-icon-badge-muted mb-4 h-11 w-11 rounded-lg">
              <Icon icon="carbon:chat" class="h-5 w-5" />
            </div>
            <p class="text-sm font-medium" style="color: var(--text-strong);">
              {searchTemporaryValue ? 'No results found' : 'No conversations'}
            </p>
            <p class="mt-1.5 max-w-xs text-xs leading-5" style="color: var(--text-muted);">
              {searchTemporaryValue ? 'Try a different search term.' : 'Start a new conversation to begin sending messages.'}
            </p>
            {#if !searchTemporaryValue}
              <button
                onclick={createNewMessage}
                class="shell-button shell-button-primary mt-4"
              >
                <Icon icon="carbon:add" class="h-3.5 w-3.5" />
                New Conversation
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .conversation-scroll-region {
    -webkit-overflow-scrolling: touch;
    touch-action: pan-y;
    overscroll-behavior-y: contain;
  }

  .line-clamp-1 {
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
