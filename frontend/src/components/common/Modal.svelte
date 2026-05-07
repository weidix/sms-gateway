<!-- frontend/src/lib/components/common/Modal.svelte -->
<script>
    import { fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    
    let { 
        isOpen = false, 
        onClose = () => {},
        maxWidth = "max-w-4xl",
        overlayClass = "",
        class: className = "",
        children
    } = $props();

    function handleKeydown(e) {
        if (e.key === 'Escape') onClose();
    }
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
        class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-black/28 px-3 py-4 backdrop-blur-md sm:px-4 sm:py-6 {overlayClass}"
        transition:fade={{ duration: 200 }}
        onclick={(e) => {
            if (e.target === e.currentTarget) onClose();
        }}
        onkeydown={handleKeydown}
        role="dialog"
        aria-modal="true"
        tabindex="0"
    >
        <div
            class="my-auto flex w-full {maxWidth} max-h-[calc(100dvh-2rem)] flex-col overflow-hidden rounded-[32px] sm:max-h-[calc(100dvh-3rem)] {className}"
            style="background: linear-gradient(180deg, var(--bg-canvas), var(--bg-shell)); border: 1px solid var(--line-soft); box-shadow: var(--shadow-strong);"
            transition:fade={{ delay: 100, duration: 300, easing: cubicOut }}
        >
            {@render children?.()}
        </div>
    </div>
{/if}
