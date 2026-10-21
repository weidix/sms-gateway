<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Button from '$lib/components/Button.svelte';
  import Field from '$lib/components/Field.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Input from '$lib/components/Input.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { auth } from '$lib/stores/auth.svelte.js';

  let username = $state('');
  let password = $state('');
  let remember = $state(false);
  let error = $state('');
  let checking = $state(true);

  onMount(async () => {
    await auth.init();
    if (!auth.loginRequired || auth.sessionValid) {
      goto('/', { replaceState: true });
      return;
    }
    checking = false;
  });

  async function handleSubmit(event) {
    event.preventDefault();
    error = '';
    try {
      await auth.signIn(username.trim(), password, remember);
      goto('/', { replaceState: true });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Sign in failed.';
    }
  }
</script>

<svelte:head>
  <title>Sign in — SMS Gateway</title>
</svelte:head>

<div class="login">
  <main class="login__card">
    <Logo size={32} showWordmark={false} />

    <div class="login__intro">
      <h1>Sign in</h1>
      <p>Use the credentials configured in <span class="mono">config.toml</span>.</p>
    </div>

    {#if checking}
      <p class="login__status">Checking the gateway…</p>
    {:else}
      <form class="login__form" onsubmit={handleSubmit}>
        <Field label="Username" forId="login-username">
          <Input
            id="login-username"
            bind:value={username}
            autocomplete="username"
            disabled={auth.busy}
          />
        </Field>

        <Field label="Password" forId="login-password">
          <Input
            id="login-password"
            type="password"
            bind:value={password}
            autocomplete="current-password"
            disabled={auth.busy}
          />
        </Field>

        <label class="remember">
          <input type="checkbox" bind:checked={remember} disabled={auth.busy} />
          <span>Keep me signed in</span>
        </label>

        {#if error}
          <p class="login__error" role="alert">
            <Icon name="alert" size={13} />
            <span>{error}</span>
          </p>
        {/if}

        <Button
          variant="primary"
          block
          type="submit"
          loading={auth.busy}
          disabled={!username.trim() || !password}
        >
          Sign in
        </Button>
      </form>
    {/if}
  </main>
</div>

<style>
  .login {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100dvh;
    padding: var(--space-6);
    background: var(--paper);
  }

  .login__card {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    width: 100%;
    max-width: 384px;
    padding: 30px;
    border-radius: 20px;
    background: var(--raised);
    box-shadow: var(--shadow-card);
  }

  .login__intro {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .login__intro h1 {
    font-family: var(--font-sans);
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: 1.1;
    letter-spacing: -0.03em;
  }

  .login__intro p {
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .login__form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .remember {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 24px;
    color: var(--ink-2);
    font-size: var(--text-sm);
    cursor: pointer;
    user-select: none;
  }

  .remember input {
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--ink);
  }

  .login__status {
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .login__error {
    display: flex;
    align-items: flex-start;
    gap: 7px;
    margin: 0;
    color: var(--danger);
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  .login__error :global(svg) {
    margin-top: 2px;
  }

  @media (max-width: 640px) {
    .login {
      padding: 16px;
    }

    .login__card {
      padding: 24px;
      border-radius: 16px;
    }
  }
</style>
