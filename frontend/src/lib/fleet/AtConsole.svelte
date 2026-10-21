<script>
  import Panel from '$lib/components/Panel.svelte';
  import Button from '$lib/components/Button.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Input from '$lib/components/Input.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { atStatusView, formatDuration, isAtExecution, QUICK_COMMANDS } from './at.js';
  import { api } from '$lib/api.js';
  import { formatDateTime, formatTime } from '$lib/format.js';
  import { toastError, toastOk, toastWarn } from '$lib/stores/toast.svelte.js';

  let { simId } = $props();

  let entries = $state([]);
  let command = $state('');
  let running = $state(false);
  let activeCommand = $state('');
  let nextEntryId = 0;

  function pushEntry(result) {
    nextEntryId += 1;
    entries = [{ ...result, id: nextEntryId }, ...entries];
  }

  async function runCommand(raw) {
    const text = String(raw ?? '').trim();
    if (!text) {
      toastWarn('Enter an AT command first.');
      return;
    }
    if (running) return;

    running = true;
    activeCommand = text;
    try {
      const result = await api.runAt(simId, text);
      pushEntry(result);
      command = '';
    } catch (cause) {
      if (isAtExecution(cause?.data)) {
        pushEntry(cause.data);
        toastError(`Transport error while running ${cause.data.command}.`);
      } else {
        toastError(cause?.message ?? 'The AT command failed.');
      }
    } finally {
      running = false;
      activeCommand = '';
    }
  }

  function handleCommandKeydown(event) {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    runCommand(command);
  }

  async function copyText(value, message) {
    try {
      await navigator.clipboard.writeText(value);
      toastOk(message);
    } catch {
      toastError('Could not copy to the clipboard.');
    }
  }

  function copyEntry(entry) {
    copyText(entry.output || '(no output)', 'Command output copied.');
  }

  function copyTranscript() {
    const text = [...entries]
      .reverse()
      .map((entry) => `${entry.command} → ${entry.status}\n${entry.output || '(no output)'}`)
      .join('\n\n');
    copyText(text, 'Transcript copied.');
  }

  function clearTranscript() {
    entries = [];
  }
</script>

{#snippet actions()}
  <IconButton
    name="copy"
    label="Copy transcript"
    size={30}
    disabled={!entries.length}
    onclick={copyTranscript}
  />
  <IconButton
    name="trash"
    label="Clear transcript"
    size={30}
    disabled={!entries.length}
    onclick={clearTranscript}
  />
{/snippet}

<div class="console-shell">
  <Panel title="AT console" flush {actions}>
    <div class="console">
      <div class="console__presets" role="group" aria-label="Quick AT commands">
        {#each QUICK_COMMANDS as preset (preset.command)}
          <Button
            size="sm"
            disabled={running}
            title={`${preset.hint} — ${preset.command}`}
            onclick={() => runCommand(preset.command)}
          >
            <span class="mono">{preset.command}</span>
          </Button>
        {/each}
      </div>

      <div class="console__form">
        <div class="console__field">
          <Input
            id="at-command"
            bind:value={command}
            placeholder="AT+CSQ"
            disabled={running}
            onkeydown={handleCommandKeydown}
          />
        </div>
        <div class="console__run">
          <Button variant="primary" loading={running} disabled={running} onclick={() => runCommand(command)}>
            Run
          </Button>
        </div>
      </div>

      <div class="console__transcript scroll-y" role="log" aria-label="AT command transcript">
        {#if running}
          <div class="console__pending" role="status">
            <Spinner size={11} />
            <span class="mono truncate">{activeCommand}</span>
          </div>
        {/if}

        {#if entries.length}
          <ol class="entries">
            {#each entries as entry (entry.id)}
              <li class="entry">
                <header class="entry__head">
                  <span class="entry__command mono truncate">{entry.command}</span>
                  <Badge tone={atStatusView(entry.status).tone}>{atStatusView(entry.status).label}</Badge>
                  <span class="entry__time mono" title={formatDateTime(entry.executed_at)}>
                    {formatTime(entry.executed_at)}
                  </span>
                  <span class="entry__duration mono">{formatDuration(entry.duration_ms)}</span>
                  <IconButton
                    name="copy"
                    label={`Copy output of ${entry.command}`}
                    size={30}
                    onclick={() => copyEntry(entry)}
                  />
                </header>
                {#if entry.output}
                  <pre class="entry__output mono">{entry.output}</pre>
                {:else}
                  <p class="entry__empty">No output.</p>
                {/if}
              </li>
            {/each}
          </ol>
        {:else if !running}
          <div class="console__empty">
            <EmptyState icon="terminal" title="No commands yet" />
          </div>
        {/if}
      </div>
    </div>
  </Panel>
</div>

<style>
  /* Let the console panel fill the detail column so both column edges align. */
  .console-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .console-shell > :global(.panel) {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
  }

  .console-shell > :global(.panel) > :global(.panel__body) {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
  }

  .console {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    gap: var(--space-3);
    padding: 13px 15px 15px;
    min-width: 0;
    min-height: 0;
  }

  .console__presets {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .console__form {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .console__field {
    flex: 1;
    min-width: 0;
  }

  .console__run {
    display: flex;
    align-items: center;
    flex: none;
  }

  /* Command input is 34px high; match the button so the row shares one height. */
  .console__run :global(.btn) {
    height: 36px;
    min-width: 68px;
  }

  .console__transcript {
    flex: 1 1 auto;
    min-height: 320px;
    padding-top: var(--space-1);
    min-width: 0;
  }

  .console__pending {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: var(--space-2);
    padding: 0 2px;
    font-size: var(--text-xs);
    color: var(--ink-3);
  }

  .entries {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    list-style: none;
  }

  .entry {
    display: flex;
    flex-direction: column;
    gap: 7px;
    padding: 11px 13px;
    border: 0;
    border-radius: 12px;
    background: var(--surface);
    min-width: 0;
  }

  .entry__head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .entry__command {
    flex: 1 1 90px;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--ink);
  }

  .entry__time,
  .entry__duration {
    flex: none;
    font-size: var(--text-2xs);
    color: var(--ink-3);
    white-space: nowrap;
  }

  .entry__output {
    margin: 0;
    max-height: 180px;
    overflow: auto;
    padding: 8px 10px;
    border-radius: 9px;
    background: var(--raised);
    font-size: var(--text-xs);
    line-height: 1.5;
    color: var(--ink-2);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .entry__empty {
    font-size: var(--text-xs);
    color: var(--ink-3);
  }

  .console__empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }


  @container page (max-width: 640px) {
    .entry__time {
      display: none;
    }
  }
</style>
