<script>
  import { fly } from "svelte/transition";
  import { quartOut } from "svelte/easing";
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
          in:fly={{ y: 28, duration: 420, easing: quartOut }}
          out:fly={{ y: -18, duration: 240, easing: quartOut }}
        >
          <Dashboard />
        </div>
      {:else}
        <div
          class="h-full"
          in:fly={{ y: 24, duration: 420, easing: quartOut }}
          out:fly={{ y: -18, duration: 240, easing: quartOut }}
        >
          <Login />
        </div>
      {/if}
    </div>
  {/if}
</div>
