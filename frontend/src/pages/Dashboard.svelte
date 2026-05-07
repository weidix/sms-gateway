<script>
  import { updateStorageValue } from "../js/storage";
  import Sidebar from "../components/layout/Sidebar.svelte";
  import MessageList from "../components/message/MessageList.svelte";
  import SimCardModal from "../components/sim-card/SimCardModal.svelte";
  import Icon from "@iconify/svelte";

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

<div class="relative flex h-full w-full overflow-hidden p-2.5 sm:p-3.5 lg:gap-3.5">
  {#if isSidebarOpen}
    <div
      class="fixed inset-0 z-20 bg-black/30 backdrop-blur-md lg:hidden"
      role="button"
      tabindex="0"
      aria-label="Close sidebar"
      onclick={closeSidebar}
      onkeydown={handleOverlayKeydown}
    ></div>
  {/if}

  <div
    class={`fixed inset-y-2.5 left-0 z-30 w-screen max-w-sm transform transition-transform duration-300 ease-out sm:left-2.5 sm:w-[calc(100vw-1.25rem)] lg:static lg:w-[20rem] lg:translate-x-0
      ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}`}
  >
    <Sidebar 
      onSimCardClick={handleSimCardClick}
      onLogoutClick={logout}
      onConversationSelect={closeSidebar}
    />
  </div>

  <div class="shell-card relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
    <div class="flex items-center justify-between border-b px-4 py-2.5 lg:hidden"
      style="border-color: var(--line-soft); background: linear-gradient(180deg, var(--panel-strong), rgba(255,255,255,0));"
    >
      <button
        class="shell-button h-10 px-3 py-2"
        onclick={toggleSidebar}
      >
        <Icon icon={isSidebarOpen ? "carbon:close" : "carbon:menu"} class="h-5 w-5" />
        <span class="text-sm">{isSidebarOpen ? 'Close' : 'Conversations'}</span>
      </button>

      <button
        class="shell-button h-10 w-10 px-0"
        onclick={handleSimCardClick}
        aria-label="Open SIM details"
      >
        <Icon icon="carbon:sim-card" class="h-5 w-5" />
      </button>
    </div>

    <div class="flex min-h-0 flex-1 flex-col">
      <MessageList />
    </div>
  </div>
</div>

<SimCardModal
  isOpen={modemInfoCardOpen}
  onClose={handleModalClose}
/>
