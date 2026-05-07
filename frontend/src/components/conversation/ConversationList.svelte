<script>
  import Icon from "@iconify/svelte";
  import { formatDate } from "../../js/dateFormat";
  import { flip } from "svelte/animate";
  import { fade, slide, scale } from "svelte/transition";
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
  <div class="mb-3 flex flex-col gap-2.5">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="shell-label">Thread Index</p>
        <h2 class="shell-heading text-xl font-semibold">Messages</h2>
      </div>
      <button
        class="shell-button shell-button-primary h-10 w-10 px-0"
        onclick={createNewMessage}
        title="New Message"
      >
        <Icon icon="carbon:add" class="h-4 w-4" />
      </button>
    </div>

    <div class="relative">
      <Icon
        icon="carbon:search"
        class="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2"
        style="color: var(--text-muted);"
      />
      <input
        type="text"
        onfocus={searchHandleFocus}
        onblur={searchHandleBlur}
        bind:value={searchTemporaryValue}
        class={`shell-input h-10 pl-10 pr-10 ${searchTemporaryIsActive ? 'border-[color:var(--line-strong)]' : ''}`}
        placeholder="Search conversations..."
      />
      {#if searchTemporaryValue}
        <button
          onclick={() => searchTemporaryValue = ""}
          class="absolute right-2 top-1/2 flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
          aria-label="Clear search"
        >
          <Icon icon="carbon:close" class="h-4 w-4" style="color: var(--text-muted);" />
        </button>
      {/if}
    </div>

    <div class="flex items-center justify-between text-xs">
      <span style="color: var(--text-muted);">
        {filteredConversations.length} visible conversation{filteredConversations.length === 1 ? '' : 's'}
      </span>
      <span class="shell-chip shell-chip-muted">
        Active relay
      </span>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-hidden">
    <div class="conversation-scroll-region shell-scrollbar h-full overflow-y-auto pr-1">
      <div class="space-y-1.5 pb-2">
        {#if $conversationLoading}
          {#each Array(5) as _}
            <div class="shell-card-muted animate-pulse p-2.5">
              <div class="flex items-center gap-2.5">
                <div class="h-10 w-10 rounded-[18px] bg-black/10 dark:bg-white/10"></div>
                <div class="flex-1 space-y-2">
                  <div class="h-3 w-32 rounded-full bg-black/10 dark:bg-white/10"></div>
                  <div class="h-3 w-24 rounded-full bg-black/10 dark:bg-white/10"></div>
                </div>
              </div>
            </div>
          {/each}
        {:else}
          {#each filteredConversations as conversation (conversation.contact.id)}
            {@const isCurrent = $currentContact?.id === conversation.contact.id}
            {@const hasPreviewContent = Boolean(
              conversation.sms_preview?.message || conversation.sms_preview?.sim_id
            )}
            <div
              animate:flip={{ duration: 300, easing: cubicOut }}
              transition:fade={{ duration: 200 }}
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
                class={`group relative overflow-hidden rounded-[20px] border px-2.5 py-2 transition-all duration-200 ${
                  isCurrent
                    ? 'border-[color:var(--accent-copper)] bg-[var(--paper-strong)] shadow-[var(--shadow-strong)]'
                    : 'border-transparent hover:border-[color:var(--line-soft)] hover:bg-[var(--panel-soft)]'
                }`}
              >
                {#if isCurrent}
                  <div
                    class="pointer-events-none absolute inset-[1px] rounded-[18px] border"
                    style="border-color: rgba(171, 113, 65, 0.2);"
                  ></div>
                  <div
                    class="pointer-events-none absolute inset-x-0 top-0 h-10"
                    style="background: linear-gradient(180deg, rgba(171, 113, 65, 0.08), transparent);"
                  ></div>
                {/if}

                <div class="flex items-center gap-2.5">
                  <div class="relative flex-shrink-0">
                    <div class={`flex h-10 w-10 items-center justify-center rounded-[18px] transition-all duration-200 ${isCurrent ? 'bg-[var(--ink-strong)] text-[var(--paper-strong)] shadow-[var(--shadow-strong)] ring-1 ring-[color:rgba(171,113,65,0.28)]' : 'bg-[var(--panel-strong)] text-[var(--text-secondary)] border border-[color:var(--line-soft)]'}`}>
                      <Icon icon="carbon:user-avatar" class="h-[18px] w-[18px]" />
                    </div>

                    {#if conversation.sms_preview?.status === SmsStatus.Unread}
                      <div
                        class="absolute -right-1 -top-1 h-3 w-3 rounded-full border-2"
                        style="background: var(--accent-copper); border-color: var(--panel-strong);"
                        transition:scale={{ duration: 200 }}
                      ></div>
                    {/if}
                  </div>

                  <div class="min-w-0 flex-1">
                    <div class="mb-0.5 flex items-start justify-between gap-2">
                      <div class="min-w-0">
                        <div class="flex items-center gap-2">
                          <h3 class="truncate pr-2 text-sm font-semibold" style="color: var(--text-strong);">
                            {conversation.contact.name}
                          </h3>
                        </div>
                        {#if conversation.sms_preview && hasPreviewContent}
                          <span class="mt-0.5 inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-semibold"
                            style={`background: ${isCurrent ? 'rgba(171, 113, 65, 0.12)' : 'var(--panel-strong)'}; color: ${isCurrent ? 'var(--accent-copper-strong)' : 'var(--text-secondary)'};`}
                          >
                            {getSimCardDisplayName(conversation.sms_preview.sim_id)}
                          </span>
                        {/if}
                      </div>

                      {#if !conversation.contact.new && conversation.sms_preview?.timestamp}
                        <span
                          class="shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium"
                          style={`color: ${isCurrent ? 'var(--accent-copper-strong)' : 'var(--text-muted)'}; background: ${isCurrent ? 'rgba(171, 113, 65, 0.1)' : 'transparent'};`}
                        >
                          {formatDate(conversation.sms_preview.timestamp)}
                        </span>
                      {/if}
                    </div>

                    {#if conversation.sms_preview && hasPreviewContent}
                      <p
                        class="line-clamp-1 text-[13px] leading-5"
                        style={`color: ${isCurrent ? 'var(--text-strong)' : 'var(--text-secondary)'}; opacity: ${isCurrent ? '0.88' : '1'};`}
                      >
                        {conversation.sms_preview.message}
                      </p>
                    {:else}
                      <span class="text-[13px] italic" style="color: var(--text-muted);">
                        {conversation.contact.new ? 'Recipient draft' : 'No messages yet'}
                      </span>
                    {/if}
                  </div>

                  {#if conversation.contact.new === true}
                    <button
                      class="flex h-9 w-9 items-center justify-center rounded-full opacity-0 transition-all duration-200 group-hover:opacity-100 hover:bg-black/5 dark:hover:bg-white/5"
                      onclick={(e) => {
                        e.stopPropagation();
                        deleteConversationHandleClick(conversation);
                      }}
                      title="Delete"
                    >
                      <Icon icon="carbon:trash-can" class="h-4 w-4" style="color: var(--text-muted);" />
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        {/if}

        {#if !$conversationLoading && filteredConversations.length === 0}
          <div class="shell-card-muted flex flex-col items-center justify-center px-6 py-12 text-center">
            <div class="shell-icon-badge-muted mb-4 h-14 w-14 rounded-3xl">
              <Icon icon="carbon:chat" class="h-7 w-7" />
            </div>
            <p class="text-base font-semibold" style="color: var(--text-strong);">
              {searchTemporaryValue ? 'No results found' : 'No conversations'}
            </p>
            <p class="mt-2 max-w-xs text-sm leading-6" style="color: var(--text-muted);">
              {searchTemporaryValue ? 'Try a different search term.' : 'Start a new conversation to begin sending messages.'}
            </p>
            {#if !searchTemporaryValue}
              <button
                onclick={createNewMessage}
                class="shell-button shell-button-primary mt-5"
              >
                <Icon icon="carbon:add" class="h-4 w-4" />
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
