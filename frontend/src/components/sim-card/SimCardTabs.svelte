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
    <div class="border-b px-4 py-3 sm:px-6"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        <div class="shell-scrollbar flex min-h-[2.5rem] items-center gap-2 overflow-x-auto">
            {#each simCards as simCard (simCard.id)}
                <button
                    class={`relative flex h-9 max-w-[10.75rem] shrink-0 items-center overflow-hidden rounded-full border px-3 text-sm font-medium leading-none transition-all duration-200 sm:max-w-[13rem] ${
                        activeSimId === simCard.id
                            ? 'bg-[var(--panel-strong)] border-[color:var(--line-strong)] text-[var(--text-strong)] shadow-[var(--shadow-soft)]'
                            : 'border-transparent text-[var(--text-secondary)] hover:bg-black/5 dark:hover:bg-white/5'
                    }`}
                    onclick={() => onTabSwitch(simCard.id)}
                    title={getDisplayName(simCard)}
                >
                    <span class="flex min-w-0 items-center gap-1.5">
                        <Icon icon="carbon:sim-card" class="h-3.5 w-3.5 shrink-0" />
                        <span class="min-w-0 truncate whitespace-nowrap">{getDisplayName(simCard)}</span>
                    </span>
                </button>
            {/each}
        </div>
    </div>
{/if}
