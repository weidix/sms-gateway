<!-- frontend/src/lib/components/simcard/SignalStrengthIndicator.svelte -->
<script>
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
    <div class="signal-strength-compact flex h-[0.9rem] items-end justify-center gap-[0.16rem]">
        {#each Array(3) as _, i}
            <div
                class="signal-strength-bar w-[0.2rem] shrink-0 rounded-sm"
                style={`height: ${0.38 + i * 0.17}rem; background: ${i < Math.ceil((bars / 5) * 3) ? 'var(--text-secondary)' : 'var(--line-strong)'};`}
            ></div>
        {/each}
    </div>
{:else}
    <div class="flex items-center justify-between gap-3 rounded-md border px-3 py-2.5"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        <div>
            <div class="shell-data-label">Signal Strength</div>
            <div class="mt-1 flex items-end gap-3">
                <div class="text-sm font-medium leading-none" style="color: var(--text-strong);">
                    {label}
                </div>
                <div class="signal-strength-bars flex items-end gap-1">
                    {#each Array(5) as _, i}
                        <div
                            class="rounded-sm"
                            style={`width: 0.28rem; height: ${0.5 + i * 0.18}rem; background: ${i < bars ? 'var(--text-strong)' : 'var(--line-strong)'};`}
                        ></div>
                    {/each}
                </div>
            </div>
        </div>
    </div>
{/if}
