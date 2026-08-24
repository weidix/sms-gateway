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

<div class="flex flex-col items-center justify-center rounded-md border border-dashed px-6 py-12 text-center"
    style="border-color: var(--line-strong); background: transparent;"
>
    <div class="shell-icon-badge-muted mb-4 h-11 w-11 rounded-lg">
        <Icon {icon} class="h-5 w-5" />
    </div>
    <h3 class="shell-heading text-base font-semibold">
        {title}
    </h3>
    {#if description}
        <p class="mt-1.5 max-w-sm text-sm leading-6" style="color: var(--text-muted);">
            {description}
        </p>
    {/if}
    {#if showRetry}
        <button
            class="shell-button shell-button-primary mt-4"
            onclick={() => {
                onRetry();
            }}
        >
            Try Again
        </button>
    {/if}
</div>
