<script>
  import { fade } from "svelte/transition";
  import { isAuthenticated, isAuthLoading } from "./stores/auth";
  import { initConversation } from "./stores/conversation";
  import Login from "./pages/Login.svelte";
  import Dashboard from "./pages/Dashboard.svelte";


  // 监听认证状态变化
  $effect(() => {
    if ($isAuthenticated) {
      initConversation();
    }
  });
</script>

<div class="shell-page">
  {#if !$isAuthLoading}
    <div class="h-dvh w-screen overflow-hidden">
      {#if $isAuthenticated}
        <div
          class="h-full min-h-0"
          in:fade={{ duration: 180 }}
          out:fade={{ duration: 120 }}
        >
          <Dashboard />
        </div>
      {:else}
        <div
          class="h-full"
          in:fade={{ duration: 180 }}
          out:fade={{ duration: 120 }}
        >
          <Login />
        </div>
      {/if}
    </div>
  {/if}
</div>
