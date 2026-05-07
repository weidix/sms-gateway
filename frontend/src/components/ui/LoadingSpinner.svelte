<script>
  import { fade } from "svelte/transition";

  let {
    show = false,
    duration = 150,
    size = "w-5 h-5"
  } = $props();
</script>

{#if show}
  <div
    class="absolute inset-0 z-10 flex items-center justify-center"
    transition:fade={{ duration }}
  >
    <div
      class="inline-block animate-spin rounded-full border-2 {size}"
      style="border-color: rgba(104, 114, 87, 0.18); border-top-color: var(--accent-copper);"
    ></div>
  </div>
{/if}
