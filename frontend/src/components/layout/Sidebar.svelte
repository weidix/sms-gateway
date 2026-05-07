<script>
  import Icon from "@iconify/svelte";
  import ConversationList from "../conversation/ConversationList.svelte";

  let { onSimCardClick = () => {}, onLogoutClick = () => {}, onConversationSelect = () => {} } = $props();
</script>

<div class="shell-card flex h-full w-full flex-col overflow-visible p-3 sm:p-4">
  <div class="mb-3 flex items-center justify-between gap-3">
    <div class="flex items-center gap-3">
      <div class="shell-icon-badge">
        <Icon icon="carbon:chat" class="h-5 w-5" />
      </div>
      <div>
        <p class="shell-label">Workspace</p>
        <h2 class="shell-heading text-lg font-semibold">Conversations</h2>
      </div>
    </div>
    <span class="shell-chip shell-chip-muted hidden sm:inline-flex">Relay</span>
  </div>

  <div class="min-h-0 flex-1 overflow-visible">
    <ConversationList onConversationSelect={onConversationSelect} />
  </div>

  <div class="mt-3 grid gap-2 rounded-[22px] border p-2"
    style="border-color: var(--line-soft); background: var(--panel-soft);"
  >
    <button
      class="shell-button group min-h-[72px] w-full justify-start rounded-[18px] px-3 py-2.5"
      onclick={() => onSimCardClick()}
    >
      <div class="shell-icon-badge h-9 w-9 rounded-xl">
        <Icon
          icon="carbon:sim-card"
          class="h-4 w-4"
        />
      </div>
      <div class="flex flex-1 flex-col items-start text-left">
        <span class="text-sm font-semibold leading-none mb-1" style="color: var(--text-strong);">
          SIM Cards
        </span>
        <span class="line-clamp-1 text-xs leading-none" style="color: var(--text-muted);">
          Modem health and device details
        </span>
      </div>
      <Icon
        icon="carbon:chevron-right"
        class="h-4 w-4 transition-colors duration-200 group-hover:translate-x-0.5"
        style="color: var(--text-muted);"
      />
    </button>

    <button
      class="shell-button group min-h-[72px] w-full justify-start rounded-[18px] px-3 py-2.5"
      onclick={() => onLogoutClick()}
    >
      <div class="shell-icon-badge-muted h-9 w-9 rounded-xl">
        <Icon
          icon="carbon:logout"
          class="h-4 w-4"
        />
      </div>
      <div class="flex flex-1 flex-col items-start">
        <span class="text-sm font-semibold leading-tight" style="color: var(--text-strong);">
          Logout
        </span>
        <span class="line-clamp-1 text-xs leading-tight" style="color: var(--text-muted);">
          End this local session cleanly
        </span>
      </div>
      <Icon
        icon="carbon:chevron-right"
        class="h-4 w-4 transition-colors duration-200 group-hover:translate-x-0.5"
        style="color: var(--text-muted);"
      />
    </button>
  </div>
</div>

<style>
  .line-clamp-1 {
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
