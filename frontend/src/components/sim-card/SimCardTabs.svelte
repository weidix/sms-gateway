<!-- frontend/src/lib/components/simcard/SimCardTabs.svelte -->
<script>
    let {
        simCards = [],
        activeSimId = null,
        onTabSwitch = (simId) => {},
        getDisplayName = (sim) => sim.alias || sim.phone_number || `SIM ${sim.id.slice(-8)}`
    } = $props();
</script>

{#if simCards.length > 0}
    <div class="border-b px-4 sm:px-5"
        style="border-color: var(--line-soft); background: var(--panel-strong);"
    >
        <div class="shell-scrollbar -mb-px flex min-h-[2.25rem] items-stretch gap-1 overflow-x-auto pt-1">
            {#each simCards as simCard (simCard.id)}
                <button
                    class={`relative whitespace-nowrap border-b-2 px-2.5 pb-1.5 text-[0.8125rem] font-medium leading-none transition-colors duration-100 ${
                        activeSimId === simCard.id
                            ? 'text-[var(--text-strong)]'
                            : 'border-transparent text-[var(--text-muted)] hover:text-[var(--text-secondary)]'
                    }`}
                    style={activeSimId === simCard.id ? 'border-bottom-color: var(--text-strong);' : ''}
                    onclick={() => onTabSwitch(simCard.id)}
                    title={getDisplayName(simCard)}
                >
                    <span class="shell-mono">{getDisplayName(simCard)}</span>
                </button>
            {/each}
        </div>
    </div>
{/if}
