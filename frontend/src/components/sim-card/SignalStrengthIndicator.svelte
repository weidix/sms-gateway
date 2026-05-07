<!-- frontend/src/lib/components/simcard/SignalStrengthIndicator.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let { rssi = 99, compact = false } = $props();
    
    function getSignalBars(rssi) {
        if (rssi === 99 || rssi < 2) return 0;
        if (rssi >= 20) return 5;
        if (rssi >= 15) return 4;
        if (rssi >= 10) return 3;
        if (rssi >= 5) return 2;
        return 1;
    }
    
    function getSignalStrengthLabel(bars) {
        const signalLabels = [
            "No Signal",
            "Very Poor", 
            "Poor",
            "Moderate",
            "Good",
            "Excellent",
        ];
        return signalLabels[bars];
    }
    
    const bars = $derived(getSignalBars(rssi));
    const label = $derived(getSignalStrengthLabel(bars));
</script>

{#if compact}
    <div class="flex items-center gap-0.5">
        {#each Array(5) as _, i}
            <div
                class="rounded-full"
                style={`width: 0.22rem; height: ${0.35 + i * 0.12}rem; background: ${i < bars ? 'var(--accent-copper)' : 'rgba(104, 114, 87, 0.18)'};`}
            ></div>
        {/each}
    </div>
{:else}
    <div class="shell-data-row rounded-[20px] border px-4 py-3"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        <div class="shell-icon-badge-muted h-10 w-10 rounded-xl">
            <Icon icon="mage:chart-up-b" class="h-4 w-4" />
        </div>
        <div class="flex-grow">
            <div class="shell-data-label">Signal Strength</div>
            <div class="mt-1 flex items-center gap-3">
                <div class="text-sm font-medium" style="color: var(--text-strong);">
                    {label}
                </div>
                <div class="flex items-end gap-1">
                    {#each Array(5) as _, i}
                        <div
                            class="rounded-full"
                            style={`width: 0.28rem; height: ${0.5 + i * 0.18}rem; background: ${i < bars ? 'var(--accent-copper)' : 'rgba(104, 114, 87, 0.18)'};`}
                        ></div>
                    {/each}
                </div>
            </div>
        </div>
    </div>
{/if}
