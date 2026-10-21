<script>
  let {
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    disabled = false,
    invalid = false,
    id = '',
    name = '',
    autocomplete = 'off',
    inputmode = undefined,
    maxlength = undefined,
    onkeydown,
    oninput
  } = $props();
</script>

<input
  class="input"
  class:input--invalid={invalid}
  {type}
  {placeholder}
  {disabled}
  {id}
  {name}
  {autocomplete}
  {inputmode}
  {maxlength}
  bind:value
  {onkeydown}
  {oninput}
/>

<style>
  .input {
    display: block;
    width: 100%;
    height: 36px;
    padding: 0 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--raised);
    color: var(--ink);
    font-size: var(--text-base);
    line-height: 1;
    transition:
      border-color var(--dur) ease,
      box-shadow var(--dur) ease;
  }

  .input::placeholder {
    color: var(--muted);
  }

  .input:hover:not(:disabled) {
    border-color: var(--line-strong);
  }

  .input:focus {
    outline: none;
    border-color: var(--ink);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--ink) 10%, transparent);
  }

  .input:disabled {
    background: var(--sunken);
    color: var(--ink-3);
    cursor: not-allowed;
  }

  .input--invalid {
    border-color: var(--danger);
  }
</style>
