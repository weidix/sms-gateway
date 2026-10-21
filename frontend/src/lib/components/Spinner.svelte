<script>
  let { size = 14, label = 'Loading' } = $props();
</script>

<span class="spinner" style="--size: {size}px" role="status" aria-label={label}></span>

<style>
  .spinner {
    width: var(--size);
    height: var(--size);
    flex: none;
    display: inline-block;
    border-radius: 50%;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    animation: spin 700ms linear infinite;
    opacity: 0.85;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
