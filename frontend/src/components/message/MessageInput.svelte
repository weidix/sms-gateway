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
  style="border-color: var(--line-soft); background: var(--panel-strong);"
>
  <div
    class="mx-auto flex max-w-5xl flex-col gap-2 px-3 py-2.5 sm:px-6"
    style="padding-bottom: calc(0.625rem + env(safe-area-inset-bottom, 0px));"
  >
    <div class="flex items-center gap-2">
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
        class="shell-input h-10 flex-1"
      />
      <button
        onclick={handleSendClick}
        disabled={(showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim()}
        class={`shell-button h-10 shrink-0 px-4 ${(showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim() ? 'cursor-not-allowed opacity-50' : 'shell-button-primary'}`}
      >
        <Icon icon="carbon:send-filled" class="h-4 w-4" />
        <span>Send</span>
      </button>
    </div>

    <div class="flex flex-col gap-3 md:flex-row xl:w-auto xl:shrink-0">
      <div class="w-full md:w-[220px]">
        <SimSelector bind:selectedSim />
      </div>
      <div class="hidden items-center gap-1.5 md:flex">
        <Icon icon="carbon:information" class="h-3.5 w-3.5" style="color: var(--text-muted);" />
        <p class="text-xs" style="color: var(--text-muted);">
          {sendMessageContent.trim() ? `${sendMessageContent.length} chars · Enter to send` : "Select the SIM this message is sent from"}
        </p>
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
    <div class="p-5 sm:p-6">
      <div class="mb-5">
        <p class="shell-label">Confirm Message</p>
        <h3 class="shell-heading mt-1.5 text-lg font-semibold">
          Review before dispatch
        </h3>
      </div>

      <div class="mb-4 rounded-md border p-3" style="border-color: var(--line-soft); background: var(--panel-soft);">
        <p class="shell-data-label">Sending From</p>
        <div class="mt-1.5 flex items-center justify-between gap-3">
          <p class="text-sm font-medium" style="color: var(--text-strong);">
            {selectedSim ? selectedSim.alias : 'Not Selected'}
          </p>
          <p class="shell-mono text-xs" style="color: var(--text-muted);">
            {selectedSim ? selectedSim.phone_number : '—'}
          </p>
        </div>
      </div>

      {#if sendMessageContent}
        <div class="mb-4">
          <div class="mb-1.5 flex items-center justify-between">
            <p class="shell-data-label">Message</p>
            <span class="shell-mono text-xs" style="color: var(--text-muted);">
              {sendMessageContent.length} chars
            </span>
          </div>
          <div class="shell-scrollbar max-h-32 overflow-y-auto rounded-md border p-3" style="border-color: var(--line-soft);">
            <p class="whitespace-pre-wrap text-sm leading-6" style="color: var(--text-secondary);">
              {sendMessageContent}
            </p>
          </div>
        </div>
      {/if}

      <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
        <button onclick={cancelSend} class="shell-button w-full">
          Cancel
        </button>
        <button onclick={confirmSend} class="shell-button shell-button-primary w-full">
          <Icon icon="carbon:send-filled" class="h-3.5 w-3.5" />
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
