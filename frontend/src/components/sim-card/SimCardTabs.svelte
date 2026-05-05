<!-- frontend/src/lib/components/simcard/SimCardTabs.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let {
        simCards = [],
        activeSimId = null,
        onTabSwitch = (simId) => {},
        getDisplayName = (sim) => sim.alias || sim.phone_number || `SIM ${sim.id.slice(-8)}`
    } = $props();
</script>

{#if simCards.length > 0}
    <div class="shell-scrollbar flex gap-2 overflow-x-auto border-b px-4 py-2.5 sm:px-6"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        {#each simCards as simCard (simCard.id)}
            <button
                class={`relative min-w-0 shrink-0 overflow-hidden rounded-full border px-4 py-2 text-sm font-medium transition-all duration-200 ${
                    activeSimId === simCard.id
                        ? 'bg-[var(--panel-strong)] border-[color:var(--line-strong)] text-[var(--text-strong)] shadow-[var(--shadow-soft)]'
                        : 'border-transparent text-[var(--text-secondary)] hover:bg-black/5 dark:hover:bg-white/5'
                }`}
                onclick={() => onTabSwitch(simCard.id)}
                title={getDisplayName(simCard)}
            >
                <span class="flex min-w-0 items-center gap-1.5">
                    <Icon icon="carbon:sim-card" class="h-3.5 w-3.5 shrink-0" />
                    <span class="min-w-0 truncate">{getDisplayName(simCard)}</span>
                </span>
            </button>
        {/each}
    </div>
{/if}
