<script>
  import Icon from "@iconify/svelte";
  import SimSelector from "./SimSelector.svelte";
  import Modal from "../common/Modal.svelte";

  let {
    sendMessageContent = $bindable(""),
    showNewMessage = false,
    concatInputText = "",
    onSend = () => {},
  } = $props();

  let isComposing = $state(false);
  let selectedSim = $state(null);
  let showConfirmDialog = $state(false);
  let messageInputRef = $state(null);

  export function focusInput() {
    if (!messageInputRef || messageInputRef.disabled) return;

    messageInputRef.focus();
    messageInputRef.setSelectionRange?.(
      messageInputRef.value.length,
      messageInputRef.value.length
    );
  }

  function handleSendClick() {
    if (showNewMessage && !concatInputText.trim()) {
      return;
    }

    if (!sendMessageContent.trim()) {
      return;
    }

    if (!selectedSim) {
      alert("Please select a SIM card first");
      return;
    }

    showConfirmDialog = true;
  }

  function confirmSend() {
    if (selectedSim) {
      onSend(selectedSim.id);
      showConfirmDialog = false;
      sendMessageContent = ""; // 清空输入框
    }
  }

  function cancelSend() {
    showConfirmDialog = false;
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && !isComposing) {
      handleSendClick();
    }
  }
</script>

<div
  class="relative z-20 shrink-0 border-t"
  style="border-color: var(--line-soft); background: linear-gradient(180deg, rgba(255,255,255,0), var(--panel) 28%, var(--panel-strong));"
>
  <div
    class="mx-auto max-w-5xl px-2.5 pb-2.5 pt-2 sm:px-6 sm:pb-5 sm:pt-3"
    style="padding-bottom: calc(0.75rem + env(safe-area-inset-bottom, 0px));"
  >
    <div class="shell-card-compact flex flex-col gap-2.5 px-2.5 py-2.5 sm:gap-3 sm:px-4 sm:py-4">
      <div class="flex flex-col gap-2 px-1 sm:flex-row sm:items-start sm:justify-between">
        <div class="min-w-0">
          <p class="shell-label">Composer</p>
          <p class="text-xs font-medium sm:text-sm" style="color: var(--text-secondary);">
            {showNewMessage && !concatInputText.trim() ? "Add a recipient to unlock sending." : "Type, select a SIM, and send immediately."}
          </p>
        </div>
        <span class="shell-chip shell-chip-muted self-start sm:shrink-0">
          {sendMessageContent.trim() ? `${sendMessageContent.length} chars` : "Ready"}
        </span>
      </div>

      <div class="flex flex-col gap-3 xl:flex-row xl:items-center">
        <div class="relative min-w-0 flex-1">
          <Icon
            icon="carbon:chat"
            class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2"
            style="color: var(--text-muted);"
          />
          <input
            type="text"
            bind:value={sendMessageContent}
            bind:this={messageInputRef}
            oncompositionstart={() => (isComposing = true)}
            oncompositionend={() => (isComposing = false)}
            onkeydown={handleKeyDown}
            disabled={showNewMessage && !concatInputText.trim()}
            placeholder={showNewMessage && !concatInputText.trim()
              ? "Enter recipient first"
              : "Type your message"}
            class="shell-input h-12 rounded-[22px] pl-11 pr-4 sm:h-14 sm:rounded-[24px] sm:pl-12"
          />
        </div>

        <div class="flex flex-col gap-3 md:flex-row xl:w-auto xl:shrink-0">
          <div class="w-full md:w-[220px]">
            <SimSelector bind:selectedSim />
          </div>

          <button
            onclick={handleSendClick}
            disabled={(showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim()}
            class={`shell-button h-12 min-w-[112px] shrink-0 rounded-[22px] sm:h-14 sm:min-w-[124px] sm:rounded-[24px] ${((showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim()) ? 'cursor-not-allowed opacity-50' : 'shell-button-primary'}`}
          >
            <Icon
              icon="carbon:send-filled"
              class="h-5 w-5"
            />
            <span>Send</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<Modal 
  isOpen={showConfirmDialog} 
  onClose={cancelSend}
  maxWidth="max-w-md"
>
  {#snippet children()}
    <div class="p-6 sm:p-8">
      <div class="mb-8 text-center">
        <div class="shell-icon-badge mx-auto mb-4">
          <Icon icon="carbon:send-alt" class="h-6 w-6" />
        </div>
        <h3 class="shell-heading text-xl font-semibold sm:text-2xl">
          Confirm Message
        </h3>
        <p class="mt-2 text-sm leading-6 shell-subtitle">
          Review the route and content before dispatch.
        </p>
      </div>

      <div class="mb-6">
        <p class="shell-label mb-3">
          Sending From
        </p>
        <div class="shell-card-muted p-4">
          <div class="flex items-center gap-3">
            <div class="shell-icon-badge h-10 w-10 rounded-xl shrink-0">
              <Icon icon="carbon:sim-card" class="h-5 w-5" />
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium" style="color: var(--text-strong);">
                {selectedSim ? selectedSim.alias : 'Not Selected'}
              </p>
              <p class="mt-1 text-xs font-mono" style="color: var(--text-muted);">
                {selectedSim ? selectedSim.phone_number : '—'}
              </p>
            </div>
            <div class="flex items-center gap-1.5">
              <span class="shell-status-dot"></span>
              <span class="text-xs font-medium" style="color: var(--text-muted);">Active</span>
            </div>
          </div>
        </div>
      </div>

      {#if sendMessageContent}
        <div class="mb-6">
          <div class="flex items-center justify-between mb-3">
            <p class="shell-label">
              Message Content
            </p>
            <span class="text-xs font-mono" style="color: var(--text-muted);">
              {sendMessageContent.length} chars
            </span>
          </div>
          <div class="shell-card-muted shell-scrollbar max-h-32 overflow-y-auto p-4">
            <p class="whitespace-pre-wrap text-sm leading-7" style="color: var(--text-secondary);">
              {sendMessageContent}
            </p>
          </div>
        </div>
      {/if}

      <div class="mb-8 rounded-[22px] border p-3"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
      >
        <div class="flex items-center gap-2">
          <Icon icon="carbon:information" class="h-4 w-4 shrink-0" style="color: var(--text-muted);" />
          <p class="text-xs leading-relaxed" style="color: var(--text-secondary);">
            Standard SMS rates apply. Dispatch is immediate once confirmed.
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <button
          onclick={cancelSend}
          class="shell-button w-full"
        >
          Cancel
        </button>
        <button
          onclick={confirmSend}
          class="shell-button shell-button-primary w-full"
        >
          <Icon icon="carbon:send-filled" class="h-4 w-4" />
          <span>Send</span>
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style lang="postcss">
  .whitespace-pre-wrap {
    white-space: pre-wrap;
  }
</style>
