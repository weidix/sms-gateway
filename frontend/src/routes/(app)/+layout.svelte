<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '$lib/components/Icon.svelte';
  import Button from '$lib/components/Button.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { auth } from '$lib/stores/auth.svelte.js';

  let { children } = $props();
  let shellReady = $state(false);

  const navigation = [
    { href: '/', label: 'Inbox', icon: 'message' },
    { href: '/fleet', label: 'Fleet', icon: 'radio' }
  ];

  const activePath = $derived(
    page.url.pathname === '/'
      ? '/'
      : page.url.pathname.startsWith('/fleet')
        ? '/fleet'
        : ''
  );

  onMount(async () => {
    await auth.init();
    if (auth.loginRequired && !auth.sessionValid) {
      goto('/login', { replaceState: true });
      return;
    }
    shellReady = true;
  });
</script>

<div class="shell">
  <header class="topbar">
    <a class="topbar__brand" href="/" aria-label="SMS Gateway inbox">
      <Logo size={30} />
    </a>

    <nav class="topbar__nav" aria-label="Primary navigation">
      {#each navigation as item (item.href)}
        <a
          class="tab"
          class:tab--active={activePath === item.href}
          href={item.href}
          aria-current={activePath === item.href ? 'page' : undefined}
        >
          <Icon name={item.icon} size={15} />
          <span>{item.label}</span>
        </a>
      {/each}
    </nav>

    {#if auth.loginRequired}
      <div class="topbar__end">
        <Button variant="ghost" size="sm" onclick={() => auth.signOut()}>Sign out</Button>
      </div>
    {/if}
  </header>

  <div class="shell__main">
    {#if shellReady}
      {@render children()}
    {/if}
  </div>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    min-height: 0;
    overflow: hidden;
    background: var(--paper);
  }

  .topbar {
    view-transition-name: topbar;
    position: relative;
    z-index: 10;
    display: flex;
    align-items: center;
    height: var(--topbar-h);
    flex: none;
    padding: 0 32px;
    background: var(--surface);
    box-shadow: inset 0 -1px 0 var(--line);
  }

  .topbar__brand {
    display: flex;
    align-items: center;
    margin-right: 20px;
    border-radius: 10px;
  }

  .topbar__nav {
    display: flex;
    align-items: stretch;
    align-self: stretch;
    gap: 2px;
  }

  .tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    align-self: stretch;
    padding: 0 14px;
    color: var(--ink-3);
    font-size: var(--text-sm);
    font-weight: 500;
    transition:
      color var(--dur) ease,
      background var(--dur) ease;
  }

  .tab:hover {
    color: var(--ink);
    background: color-mix(in srgb, var(--ink) 4%, transparent);
  }

  .tab--active {
    color: var(--ink);
    font-weight: 600;
  }

  .tab--active::after {
    content: '';
    position: absolute;
    left: 14px;
    right: 14px;
    bottom: 0;
    height: 2px;
    background: var(--ink);
    border-radius: 2px 2px 0 0;
  }

  .topbar__end {
    display: flex;
    align-items: center;
    margin-left: auto;
  }

  .shell__main {
    view-transition-name: page;
    background: var(--paper);
    flex: 1;
    min-height: 0;
    height: calc(100dvh - var(--topbar-h));
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
  }

  @media (max-width: 760px) {
    .topbar {
      padding: 0 16px;
      gap: 2px;
    }

    .topbar__brand {
      margin-right: 6px;
    }

    .tab {
      gap: 6px;
      padding: 0 9px;
      font-size: var(--text-xs);
    }

    .tab--active::after {
      left: 9px;
      right: 9px;
    }
  }
</style>
