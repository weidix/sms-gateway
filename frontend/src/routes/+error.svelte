<script>
  import { page } from '$app/state';
  import Button from '$lib/components/Button.svelte';

  const status = $derived(page.status ?? 500);
  const message = $derived(page.error?.message ?? 'Something went wrong while opening this screen.');
</script>

<svelte:head>
  <title>{status} — SMS Gateway</title>
</svelte:head>

<div class="error-page">
  <div class="error-card">
    <span class="eyebrow">Error {status}</span>
    <h1>{status === 404 ? 'This screen does not exist.' : 'The console hit a problem.'}</h1>
    {#if status !== 404}
      <p>{message}</p>
    {/if}
    <div class="error-card__actions">
      <Button variant="primary" href="/">Back to workspace</Button>
      <Button variant="ghost" href="/fleet">Open fleet</Button>
    </div>
  </div>
</div>

<style>
  .error-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100dvh;
    padding: var(--space-6);
    background: var(--paper);
  }

  .error-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    width: 100%;
    max-width: 460px;
    padding: var(--space-6);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .error-card h1 {
    font-family: var(--font-serif);
    font-size: var(--text-xl);
    font-weight: 500;
    letter-spacing: -0.015em;
  }

  .error-card p {
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  .error-card__actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
</style>
