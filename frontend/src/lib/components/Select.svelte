<script>
  import { Select as SelectPrimitive } from 'bits-ui';
  import Icon from './Icon.svelte';

  let {
    value = $bindable(''),
    options = [],
    placeholder = 'Select…',
    disabled = false,
    label = '',
    id = '',
    invalid = false
  } = $props();

  const selectedLabel = $derived(options.find((option) => option.value === value)?.label ?? '');
</script>

<div class="select" class:select--invalid={invalid}>
  <SelectPrimitive.Root type="single" bind:value {disabled}>
    <SelectPrimitive.Trigger
      class="select__trigger"
      aria-label={label || placeholder}
      {id}
    >
      <span class="select__value" class:select__value--empty={!selectedLabel}>
        {selectedLabel || placeholder}
      </span>
      <Icon name="chevron-down" size={13} />
    </SelectPrimitive.Trigger>

    <SelectPrimitive.Portal>
      <SelectPrimitive.Content class="select__content" sideOffset={4} align="start">
        <SelectPrimitive.Viewport class="select__viewport">
          {#each options as option (option.value)}
            <SelectPrimitive.Item
              class="select__item"
              value={option.value}
              label={option.label}
              disabled={option.disabled}
            >
              <span class="select__label">{option.label}</span>
              {#if option.value === value}
                <Icon name="check" size={13} />
              {/if}
            </SelectPrimitive.Item>
          {/each}
        </SelectPrimitive.Viewport>
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  </SelectPrimitive.Root>
</div>

<style>
  :global(.select) {
    width: 100%;
    min-width: 0;
  }

  :global(.select__trigger) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    height: 36px;
    padding: 0 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--raised);
    color: var(--ink);
    font-size: var(--text-base);
    text-align: left;
    transition:
      border-color var(--dur) ease,
      box-shadow var(--dur) ease;
  }

  :global(.select__trigger:hover) {
    border-color: var(--line-strong);
  }

  :global(.select__trigger:focus-visible) {
    outline: none;
    border-color: var(--ink);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--ink) 10%, transparent);
  }

  :global(.select--invalid .select__trigger) {
    border-color: var(--danger);
  }

  :global(.select__value) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.select__value--empty) {
    color: var(--muted);
  }

  :global(.select__content) {
    z-index: 80;
    min-width: var(--bits-select-anchor-width, 180px);
    max-width: min(360px, calc(100vw - 24px));
    max-height: min(300px, var(--bits-select-content-available-height, 300px));
    overflow: hidden;
    padding: 4px;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: var(--raised);
    box-shadow: var(--shadow-pop);
  }

  :global(.select__viewport) {
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
  }

  :global(.select__item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 30px;
    padding: 0 8px;
    border-radius: 9px;
    color: var(--ink-2);
    font-size: var(--text-base);
    cursor: default;
    user-select: none;
    outline: none;
  }

  :global(.select__item[data-highlighted]) {
    background: var(--sunken);
    color: var(--ink);
  }

  :global(.select__item[data-disabled]) {
    opacity: 0.45;
  }

  :global(.select__label) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
