<script>
  import { updateStorageValue } from "../js/storage";
  import Sidebar from "../components/layout/Sidebar.svelte";
  import MessageList from "../components/message/MessageList.svelte";
  import SimCardModal from "../components/sim-card/SimCardModal.svelte";
  import Icon from "@iconify/svelte";
  import { currentContact } from "../stores/conversation";

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

<div class="flex h-screen w-screen flex-col lg:flex-row font-sans dark:bg-zinc-900 dark:text-white relative">
  {#if isSidebarOpen}
    <div
      class="fixed inset-0 bg-black/40 backdrop-blur-sm z-20 lg:hidden"
      role="button"
      tabindex="0"
      aria-label="Close sidebar"
      onclick={closeSidebar}
      onkeydown={handleOverlayKeydown}
    ></div>
  {/if}

  <div
    class={`fixed inset-y-0 left-0 z-30 transform transition-transform duration-300 ease-out lg:relative lg:translate-x-0 lg:w-80 w-[84vw] max-w-sm
      ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}`}
  >
    <Sidebar 
      onSimCardClick={handleSimCardClick}
      onLogoutClick={logout}
      onConversationSelect={closeSidebar}
    />
  </div>
  
  <!-- 主内容区域 -->
  <div class="flex-1 bg-white dark:bg-zinc-900 flex flex-col">
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200/70 dark:border-zinc-800/70 lg:hidden sticky top-0 z-10 bg-white/90 dark:bg-zinc-900/90">
      <button
        class="inline-flex items-center gap-2 px-3 py-2 rounded-xl border border-gray-200 dark:border-zinc-700 text-slate-800 dark:text-gray-100 bg-white shadow-sm dark:bg-zinc-800 active:scale-[0.98] transition"
        onclick={toggleSidebar}
      >
        <Icon icon={isSidebarOpen ? "carbon:close" : "carbon:menu"} class="w-5 h-5" />
        <span class="text-sm">{isSidebarOpen ? 'Close' : 'Conversations'}</span>
      </button>
      <div class="flex items-center gap-3">
        <p class="text-sm text-gray-700 dark:text-gray-200 truncate max-w-[140px]">
          {$currentContact ? $currentContact.name : 'Messages'}
        </p>
        <button
          class="inline-flex items-center justify-center w-10 h-10 rounded-xl border border-gray-200 dark:border-zinc-700 text-slate-800 dark:text-gray-100 bg-white shadow-sm dark:bg-zinc-800 active:scale-[0.98] transition"
          onclick={handleSimCardClick}
          aria-label="Open SIM details"
        >
          <Icon icon="carbon:sim-card" class="w-5 h-5" />
        </button>
      </div>
    </div>
    <div class="flex-1 bg-white dark:bg-zinc-900">
      <MessageList />
    </div>
  </div>
</div>

<SimCardModal
  isOpen={modemInfoCardOpen}
  onClose={handleModalClose}
/>
