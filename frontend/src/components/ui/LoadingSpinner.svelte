<script>
  import { fade } from "svelte/transition";

  let {
    show = false,
    duration = 150,
    size = "w-4 h-4"
  } = $props();
</script>

{#if show}
  <div
    class="absolute inset-0 z-10 flex items-center justify-center"
    transition:fade={{ duration }}
  >
    <div
      class="inline-block animate-spin rounded-full border-2 {size}"
      style="border-color: var(--line-strong); border-top-color: var(--text-muted);"
    ></div>
  </div>
{/if}
