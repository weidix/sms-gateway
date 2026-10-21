<script>
  let {
    value = $bindable(''),
    placeholder = '',
    rows = 4,
    disabled = false,
    invalid = false,
    id = '',
    maxlength = undefined,
    onkeydown,
    oninput
  } = $props();
</script>

<textarea
  class="textarea"
  class:textarea--invalid={invalid}
  {rows}
  {placeholder}
  {disabled}
  {id}
  {maxlength}
  bind:value
  {onkeydown}
  {oninput}
></textarea>

<style>
  .textarea {
    display: block;
    width: 100%;
    min-height: 84px;
    padding: 9px 10px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--raised);
    color: var(--ink);
    font-size: var(--text-base);
    line-height: 1.55;
    resize: vertical;
    transition: border-color var(--dur) ease;
  }

  .textarea::placeholder {
    color: var(--muted);
  }

  .textarea:hover:not(:disabled) {
    border-color: var(--line-strong);
  }

  .textarea:focus {
    outline: none;
    border-color: var(--ink);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--ink) 10%, transparent);
  }

  .textarea:disabled {
    background: var(--sunken);
    cursor: not-allowed;
  }

  .textarea--invalid {
    border-color: var(--danger);
  }
</style>
