<!-- frontend/src/lib/components/simcard/EmptyState.svelte -->
<script>
    import Icon from "@iconify/svelte";

    let {
        icon = "mage:memory-card",
        title = "No Data",
        description = "",
        showRetry = false,
        onRetry = () => {},
    } = $props();
</script>

<div class="shell-card-muted flex flex-col items-center justify-center px-6 py-12 text-center">
    <div class="shell-icon-badge-muted mb-4 h-14 w-14 rounded-3xl">
        <Icon {icon} class="h-7 w-7" />
    </div>
    <h3 class="shell-heading text-xl font-semibold">
        {title}
    </h3>
    {#if description}
        <p class="mt-2 text-sm leading-6" style="color: var(--text-muted);">
            {description}
        </p>
    {/if}
    {#if showRetry}
        <button
            class="shell-button shell-button-primary mt-5"
            onclick={() => {
                onRetry();
            }}
        >
            Try Again
        </button>
    {/if}
</div>
