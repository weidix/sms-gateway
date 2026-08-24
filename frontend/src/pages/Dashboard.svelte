<script>
  import { updateStorageValue } from "../js/storage";
  import Sidebar from "../components/layout/Sidebar.svelte";
  import MessageList from "../components/message/MessageList.svelte";
  import SimCardModal from "../components/sim-card/SimCardModal.svelte";
  import Icon from "@iconify/svelte";
  import { isLoginRequired } from "../stores/auth";

  let modemInfoCardOpen = $state(false);
  let isSidebarOpen = $state(false);

  const logout = async () => {
    await updateStorageValue("auth", null);
    window.location.reload();
  };

  function handleSimCardClick() {
    modemInfoCardOpen = true;
    isSidebarOpen = false;
  }

  function handleModalClose() {
    modemInfoCardOpen = false;
  }

  const toggleSidebar = () => {
    isSidebarOpen = !isSidebarOpen;
  };

  const closeSidebar = () => {
    isSidebarOpen = false;
  };

  function handleOverlayKeydown(event) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      closeSidebar();
    }
  }
</script>

<div class="relative flex h-full w-full overflow-hidden">
  {#if isSidebarOpen}
    <div
      class="fixed inset-0 z-20 bg-black/50 lg:hidden"
      role="button"
      tabindex="0"
      aria-label="Close sidebar"
      onclick={closeSidebar}
      onkeydown={handleOverlayKeydown}
    ></div>
  {/if}

  <aside
    class={`fixed inset-y-0 left-0 z-30 w-screen max-w-sm transform transition-transform duration-200 ease-out sm:w-[calc(100vw-1.25rem)] lg:static lg:w-[19rem] lg:translate-x-0
      ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}`}
  >
    <div
      class="h-full w-full border-r bg-[var(--panel)]"
      style="border-color: var(--line-soft);"
    >
      <Sidebar
        onSimCardClick={handleSimCardClick}
        onLogoutClick={logout}
        showLogout={$isLoginRequired}
        onConversationSelect={closeSidebar}
      />
    </div>
  </aside>

  <main class="shell-panel relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
    <div
      class="flex items-center justify-between gap-2 border-b px-3 py-2 lg:hidden"
      style="border-color: var(--line-soft); background: var(--panel-strong);"
    >
      <button
        class="shell-button h-9 min-w-0 max-w-[calc(100%-3.5rem)] px-3 py-1.5"
        onclick={toggleSidebar}
      >
        <Icon icon={isSidebarOpen ? "carbon:close" : "carbon:menu"} class="h-4 w-4" />
        <span class="truncate text-xs font-medium sm:text-sm">{isSidebarOpen ? 'Close' : 'Conversations'}</span>
      </button>

      <button
        class="shell-button h-9 w-9 px-0"
        onclick={handleSimCardClick}
        aria-label="Open SIM details"
      >
        <Icon icon="carbon:sim-card" class="h-4 w-4" />
      </button>
    </div>

    <div class="flex min-h-0 flex-1 flex-col">
      <MessageList />
    </div>
  </main>
</div>

<SimCardModal
  isOpen={modemInfoCardOpen}
  onClose={handleModalClose}
/>
