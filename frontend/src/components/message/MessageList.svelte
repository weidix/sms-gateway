<script>
  import Icon from "@iconify/svelte";
  import { get } from "svelte/store";
  import { apiClient } from "../../js/api";
  import { formatTimeRange, formatDate } from "../../js/dateFormat";
  import {
    currentContact,
    conversationLoading,
    conversations,
    changeCurrentConversation,
    newMessageConcatChange,
    conactAddFinish,
    markConversationAsRead,
    SmsStatus,
    updateConversationLastMessage,
    deleteConversation,
  } from "../../stores/conversation";
  import { fade } from "svelte/transition";
  import { onDestroy, onMount, tick } from "svelte";
  import MessageHeader from "./MessageHeader.svelte";
  import MessageItem from "./MessageItem.svelte";
  import MessageInput from "./MessageInputOptimized.svelte";
  import {
    MessageScrollAlignment,
    getBottomAlignmentRequest,
    scrollContainerToBottom,
  } from "./messageScroll.js";
  import LoadingSpinner from "../ui/LoadingSpinner.svelte";
  import Modal from "../common/Modal.svelte";

  let messages = $state([]);
  let showNewMessage = $state(false);
  let concatInputText = $state("");
  let isAddingContact = $state(false);
  let loading = $state(true);
  let sendMessageContent = $state("");
  let page = $state(1);
  let pageSize = $state(9999999);
  let showLoading = $state(true);
  let loadingTimer = null;
  let messageInputComponent = $state(null);
  let prevConversationId = null;
  let messageContainer = $state(null);
  let isNewMessage = $state(false);
  let showRecipientDialog = $state(false);
  let draftRecipientName = $state("");
  let recipientDialogInput = $state(null);
  let autoPromptedDraftId = $state(null);
  let pendingBottomAlignment = $state(null);
  let bottomAlignmentTimer = null;
  const loadingDuration = 150;
  const displayMessages = $derived([...messages].reverse());

  $effect(() => {
    if (!$conversationLoading) {
      if ($currentContact && $currentContact.new === true) {
        showNewMessage = true;
      } else if ($currentContact && !$currentContact.new) {
        showNewMessage = false;
      }
    }
  });

  $effect(() => {
    if (!showNewMessage || $currentContact?.new !== true) {
      autoPromptedDraftId = null;
      return;
    }

    if (
      !concatInputText.trim() &&
      $currentContact?.id &&
      autoPromptedDraftId !== $currentContact.id
    ) {
      draftRecipientName = "";
      showRecipientDialog = true;
      autoPromptedDraftId = $currentContact.id;
    }
  });

  $effect(() => {
    if (showRecipientDialog && recipientDialogInput) {
      setTimeout(() => {
        recipientDialogInput?.focus();
        recipientDialogInput?.setSelectionRange?.(
          recipientDialogInput.value.length,
          recipientDialogInput.value.length,
        );
      }, 0);
    }
  });

  $effect(() => {
    if (!isAddingContact && showNewMessage) {
      newMessageConcatChange(concatInputText);
    }
  });

  $effect(() => {
    if (!$currentContact) return;

    if ($currentContact.id === prevConversationId) return;

    prevConversationId = $currentContact.id;
    loading = true;
    requestBottomAlignment(MessageScrollAlignment.ConversationChange);

    if (!$currentContact.new) {
      apiClient
        .getSmsPaginated(page, pageSize, $currentContact.id)
        .then((res) => {
          isNewMessage = false;
          messages = res.data.data;
          loading = false;
          if (page === 1) {
            markConversationAsRead($currentContact.id);
          }
        });
    } else {
      isNewMessage = false;
      messages = [];
      loading = false;
    }
  });

  $effect(() => {
    if (!pendingBottomAlignment || showLoading || !messageContainer) {
      return;
    }

    const { behavior, delayMs } = pendingBottomAlignment;
    pendingBottomAlignment = null;
    tick().then(() => {
      clearBottomAlignmentTimer();

      if (delayMs > 0) {
        bottomAlignmentTimer = setTimeout(() => {
          scrollContainerToBottom(messageContainer, behavior);
          bottomAlignmentTimer = null;
        }, delayMs);
        return;
      }

      scrollContainerToBottom(messageContainer, behavior);
    });
  });

  $effect(() => {
    if (loading) {
      showLoading = true;
      if (loadingTimer) clearTimeout(loadingTimer);
    } else {
      if (loadingTimer) clearTimeout(loadingTimer);
      loadingTimer = setTimeout(() => {
        showLoading = false;
      }, loadingDuration);
    }
  });

  function handleAddContact(name = concatInputText) {
    const trimmed = name.trim();
    if (!trimmed) return;

    isAddingContact = true;
    conactAddFinish(trimmed);
    concatInputText = trimmed;
    if ($currentContact?.new === true) {
      currentContact.set({
        ...$currentContact,
        name: trimmed,
        new: true,
      });
    }
    showRecipientDialog = false;
    isAddingContact = false;
    setTimeout(() => {
      messageInputComponent?.focusInput?.();
    }, 0);
  }

  function handleRecipientDialogClose() {
    draftRecipientName = concatInputText || "";
    showRecipientDialog = false;

    if (!$currentContact?.new || concatInputText.trim()) {
      return;
    }

    const activeConversationId = $currentContact.id;
    const fallbackConversation = get(conversations).find(
      (conversation) => conversation.contact.id !== activeConversationId,
    );

    deleteConversation(activeConversationId);

    if (fallbackConversation) {
      changeCurrentConversation(fallbackConversation.contact);
    }
  }

  function handleRecipientConfirm() {
    handleAddContact(draftRecipientName);
  }

  function handleEditRecipient(event) {
    const shouldReset = event?.detail?.reset === true;
    draftRecipientName = shouldReset ? "" : concatInputText;
    showRecipientDialog = true;
  }

  function requestBottomAlignment(source) {
    pendingBottomAlignment = getBottomAlignmentRequest(source);
  }

  function clearBottomAlignmentTimer() {
    if (bottomAlignmentTimer) {
      clearTimeout(bottomAlignmentTimer);
      bottomAlignmentTimer = null;
    }
  }

  function handleSendMessage(simId) {
    if (sendMessageContent.trim() === "") {
      return;
    }

    // Mark as new message (to enable animations)
    isNewMessage = true;

    // Add new message
    const newMessage = {
      id: -1,
      message: sendMessageContent,
      send: true,
      timestamp: new Date(),
      status: SmsStatus.Loading,
    };

    // Add message to array
    messages = [newMessage, ...messages];

    // Clear input
    sendMessageContent = "";

    // Keep the newest bubble in view after the slide-in animation completes.
    requestBottomAlignment(MessageScrollAlignment.OutgoingMessage);

    const concat =
      $currentContact.new === true
        ? {
            id: $currentContact.id,
            name: concatInputText,
          }
        : $currentContact;

    apiClient
      .sendSms(simId, concat, newMessage.message, $currentContact.new ?? false)
      .then((res) => {
        isNewMessage = false;
        const messageId = res.data;
        messages = messages.map((msg) => {
          if (msg.id === -1 && msg.message === newMessage.message) {
            return { ...msg, status: SmsStatus.Read, id: messageId.sms_id };
          }
          return msg;
        });
      })
      .catch((err) => {
        isNewMessage = false;
        messages = messages.map((msg) => {
          if (msg.id === -1 && msg.message === newMessage.message) {
            return { ...msg, status: SmsStatus.Failed };
          }
          return msg;
        });
        console.error("发送消息失败:", err);
      })
      .finally(() => {
        updateConversationLastMessage(
          $currentContact.id,
          newMessage.message,
          simId,
          concatInputText || $currentContact.name
        );
        showNewMessage = false;
        concatInputText = "";
      });
  };

  // Add auto-update functionality
  function handleMessageUpdate(event) {
    const { messages: newMessages, silentUpdate } = event.detail;

    if (!newMessages || newMessages.length === 0) return;

    // Disable animation effects
    if (silentUpdate) {
      isNewMessage = false;
    }

    // Remove duplicates to avoid repeated messages
    const existingIds = new Set(messages.map((msg) => msg.id));
    const uniqueNewMessages = newMessages.filter(
      (msg) => !existingIds.has(msg.id)
    );

    if (uniqueNewMessages.length > 0) {
      messages = [...uniqueNewMessages, ...messages];
      requestBottomAlignment(MessageScrollAlignment.IncomingUpdate);
    }
  }

  onMount(() => {
    window.addEventListener("update-messages", handleMessageUpdate);
    window.addEventListener("open-new-recipient-dialog", handleEditRecipient);
  });

  onDestroy(() => {
    if (loadingTimer) clearTimeout(loadingTimer);
    clearBottomAlignmentTimer();
    window.removeEventListener("update-messages", handleMessageUpdate);
    window.removeEventListener("open-new-recipient-dialog", handleEditRecipient);
  });

  onDestroy(() => {
    if (loadingTimer) clearTimeout(loadingTimer);
    clearBottomAlignmentTimer();
  });
</script>

<div class="relative flex min-h-0 flex-1 flex-col">
  <MessageHeader
    {showNewMessage}
    {concatInputText}
    onEditRecipient={handleEditRecipient}
  />

  <div class="relative flex-1 min-h-0 overflow-hidden">
    <LoadingSpinner show={showLoading} duration={loadingDuration} />
    {#if !showLoading}
      <div
        class="message-container shell-scrollbar flex h-full flex-col overflow-y-auto"
        bind:this={messageContainer}
        transition:fade={{ duration: loadingDuration }}
      >
        <div
          class="mx-auto mt-auto flex w-full max-w-5xl flex-col gap-3 px-3 py-3 sm:px-6 sm:py-4"
        >
          {#each displayMessages as message, index (message.id)}
            {@const previousMessage = index === 0 ? null : displayMessages[index - 1]}
            {@const timeHeader = formatTimeRange(
              message.timestamp,
              previousMessage?.timestamp ?? null
            )}
            {#if timeHeader || index === 0}
              <div
                class="my-1 flex justify-center"
                in:fade={{ duration: 300, delay: 100 }}
              >
                <span class="rounded-full px-3 py-1 text-[11px] font-medium"
                  style="background: var(--panel); color: var(--text-muted); border: 1px solid var(--line-soft);"
                >
                  {timeHeader || formatDate(message.timestamp)}
                </span>
              </div>
            {/if}
            <MessageItem {message} {isNewMessage} />
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <MessageInput
    bind:sendMessageContent
    {showNewMessage}
    {concatInputText}
    onSend={handleSendMessage}
    bind:this={messageInputComponent}
  />

  
</div>

<Modal
  isOpen={showRecipientDialog}
  onClose={handleRecipientDialogClose}
  maxWidth="max-w-md"
>
  {#snippet children()}
    <div class="p-6 sm:p-8">
      <div class="mb-7 text-center">
        <div class="shell-icon-badge mx-auto mb-4">
          <Icon icon="carbon:user-multiple-add" class="h-6 w-6" />
        </div>
        <h3 class="shell-heading text-2xl font-semibold">
          New Recipient
        </h3>
        <p class="mt-2 text-sm leading-6 shell-subtitle">
          Enter the phone number or contact name for this new thread.
        </p>
      </div>

      <div class="mb-6">
        <label for="new-recipient-input" class="shell-label mb-3 block">
          Recipient
        </label>
        <div
          class="rounded-[22px] border px-4 py-3 shadow-[var(--shadow-inset)] transition-all duration-200 focus-within:border-[color:var(--accent-copper)] focus-within:shadow-[0_0_0_1px_var(--ring-core),0_0_0_4px_var(--ring-soft)]"
          style="border-color: var(--line-soft); background: var(--panel-strong);"
        >
          <input
            id="new-recipient-input"
            type="text"
            bind:value={draftRecipientName}
            bind:this={recipientDialogInput}
            class="w-full appearance-none border-0 bg-transparent p-0 text-lg font-medium outline-none ring-0 placeholder-[color:var(--text-muted)] focus:border-0 focus:outline-none focus:ring-0 focus-visible:outline-none focus-visible:ring-0"
            style="color: var(--text-strong);"
            placeholder="Enter recipient"
            onkeydown={(event) => {
              if (event.key === "Enter") {
                handleRecipientConfirm();
              }
            }}
          />
        </div>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <button
          class="shell-button w-full"
          onclick={handleRecipientDialogClose}
        >
          Cancel
        </button>
        <button
          class={`shell-button w-full ${draftRecipientName.trim() ? "shell-button-primary" : "cursor-not-allowed opacity-50"}`}
          onclick={handleRecipientConfirm}
          disabled={!draftRecipientName.trim()}
        >
          Confirm
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .message-container {
    background:
      radial-gradient(circle at top, rgba(171, 113, 65, 0.06), transparent 32%),
      linear-gradient(180deg, rgba(255, 255, 255, 0), rgba(255, 255, 255, 0));
    -webkit-overflow-scrolling: touch;
    touch-action: pan-y;
    overscroll-behavior-y: contain;
  }

  .message-container::-webkit-scrollbar {
    width: 6px;
  }

  .message-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .message-container::-webkit-scrollbar-thumb {
    background: var(--scroll-thumb);
    border-radius: 999px;
  }

  .message-container::-webkit-scrollbar-button:start:decrement {
    height: 3rem;
    display: block;
  }

  .message-container::-webkit-scrollbar-button:end:increment {
    height: 10rem;
    display: block;
  }

</style>
