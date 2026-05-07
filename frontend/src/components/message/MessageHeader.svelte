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
  class="sticky top-0 z-10 flex min-h-[4rem] items-center border-b px-4 py-2.5 sm:px-5"
  style="border-color: var(--line-soft); background: linear-gradient(180deg, var(--panel-strong), rgba(255,255,255,0));"
>
  <div class="flex min-w-0 flex-1 items-center gap-4">
    <div class="shell-icon-badge-muted h-10 w-10 rounded-[18px]">
      <span class="text-sm font-semibold">To</span>
    </div>

    <div class="min-w-0 flex-1">
      <p class="shell-label">Recipient</p>
      {#if !$conversationLoading}
        {#if showNewMessage}
          <div class="mt-0.5 flex items-center gap-2">
            <button
              class="min-w-0 flex-1 rounded-[14px] px-0 py-1 text-left transition-opacity duration-200 hover:opacity-80"
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
              class="flex h-8 w-8 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
              onclick={onEditRecipient}
              aria-label={concatInputText ? "Edit recipient" : "Set recipient"}
            >
              <Icon
                icon={concatInputText ? "carbon:edit" : "carbon:add"}
                class="h-4 w-4"
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
