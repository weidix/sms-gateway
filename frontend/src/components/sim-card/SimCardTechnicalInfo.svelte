<!-- frontend/src/lib/components/simcard/SimCardTechnicalInfo.svelte -->
<script>
    import SignalStrengthIndicator from "./SignalStrengthIndicator.svelte";
    import SimCardEnhancedInfo from "./SimCardEnhancedInfo.svelte";
    
    let { simInfo = null } = $props();
</script>

<div class="shell-data-card space-y-4">
    <div class="flex items-center justify-between gap-3">
        <div>
            <p class="shell-label">Diagnostics</p>
            <h4 class="shell-heading mt-1 text-lg font-semibold sm:text-xl">
                Technical Details
            </h4>
        </div>
        <div class="shell-icon-badge-muted flex h-10 w-10 items-center justify-center rounded-xl">
            <SignalStrengthIndicator rssi={simInfo?.signal_quality?.rssi || 99} compact={true} />
        </div>
    </div>

    <SignalStrengthIndicator rssi={simInfo?.signal_quality?.rssi || 99} />

    <SimCardEnhancedInfo {simInfo} />
</div>
