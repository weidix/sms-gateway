<script>
  import Icon from "@iconify/svelte";
  import { currentContact, conversationLoading } from "../../stores/conversation";

  let {
    showNewMessage = false,
    concatInputText = "",
    onEditRecipient = () => {}
  } = $props();

  const recipientTitleClass =
    "mt-0.5 block truncate text-[0.95rem] font-medium sm:text-base lg:text-[1.05rem]";
</script>

<header
  class="sticky top-0 z-10 flex h-12 items-center border-b px-4 py-1.5 sm:px-5"
  style="border-color: var(--line-soft); background: var(--panel);"
>
  <div class="flex min-w-0 flex-1 items-center gap-3">
    <div class="shell-icon-badge-muted h-7 w-7 shrink-0 rounded">
      <span class="shell-mono text-[11px] font-semibold">To</span>
    </div>

    <div class="min-w-0 flex-1">
      <p class="shell-label">Recipient</p>
      {#if !$conversationLoading}
        {#if showNewMessage}
          <div class="mt-0.5 flex items-center gap-2">
            <button
              class="min-w-0 flex-1 rounded px-0 py-0.5 text-left transition-opacity duration-150 hover:opacity-70"
              onclick={onEditRecipient}
            >
              <p
                class={`${recipientTitleClass} ${
                  concatInputText ? "" : "italic"
                }`}
                style={`color: ${
                  concatInputText ? "var(--text-strong)" : "var(--text-muted)"
                };`}
              >
                {concatInputText || "Set recipient"}
              </p>
            </button>

            <button
              class="flex h-6 w-6 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
              onclick={onEditRecipient}
              aria-label={concatInputText ? "Edit recipient" : "Set recipient"}
            >
              <Icon
                icon={concatInputText ? "carbon:edit" : "carbon:add"}
                class="h-3.5 w-3.5"
                style="color: var(--text-muted);"
              />
            </button>
          </div>
        {:else if $currentContact}
          <p class={recipientTitleClass} style="color: var(--text-strong);">
            {$currentContact.name}
          </p>
        {/if}
      {/if}
    </div>
  </div>
</header>
