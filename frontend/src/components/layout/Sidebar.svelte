<script>
  import Icon from "@iconify/svelte";
  import ConversationList from "../conversation/ConversationList.svelte";

  let { onSimCardClick = () => {}, onLogoutClick = () => {}, onConversationSelect = () => {}, showLogout = true } = $props();
</script>

<div class="flex h-full w-full flex-col overflow-visible">
  <div
    class="flex items-center gap-2.5 border-b px-4 py-3"
    style="border-color: var(--line-soft);"
  >
    <div class="shell-icon-badge h-7 w-7 rounded">
      <Icon icon="carbon:send-filled" class="h-3.5 w-3.5" />
    </div>
    <div class="min-w-0 flex-1">
      <h2 class="shell-heading truncate text-sm font-semibold leading-tight">SMS Gateway</h2>
      <p class="shell-label mt-0.5 normal-case tracking-normal">Conversations</p>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-visible">
    <ConversationList onConversationSelect={onConversationSelect} />
  </div>

  <div
    class="mt-auto grid gap-px border-t"
    style="border-color: var(--line-soft); background: var(--line-soft);"
  >
    <button
      class="shell-button group h-11 w-full justify-start gap-2.5 border-0 px-4"
      onclick={() => onSimCardClick()}
    >
      <Icon icon="carbon:sim-card" class="h-4 w-4 shrink-0" style="color: var(--text-muted);" />
      <span class="flex-1 text-left text-sm font-medium" style="color: var(--text-strong);">
        SIM Cards
      </span>
      <Icon
        icon="carbon:chevron-right"
        class="h-3.5 w-3.5 transition-colors duration-150 group-hover:translate-x-0.5"
        style="color: var(--text-muted);"
      />
    </button>

    {#if showLogout}
      <button
        class="shell-button group h-11 w-full justify-start gap-2.5 border-0 px-4"
        onclick={() => onLogoutClick()}
      >
        <Icon icon="carbon:logout" class="h-4 w-4 shrink-0" style="color: var(--text-muted);" />
        <span class="flex-1 text-left text-sm font-medium" style="color: var(--text-strong);">
          Logout
        </span>
        <Icon
          icon="carbon:chevron-right"
          class="h-3.5 w-3.5 transition-colors duration-150 group-hover:translate-x-0.5"
          style="color: var(--text-muted);"
        />
      </button>
    {/if}
  </div>
</div>
