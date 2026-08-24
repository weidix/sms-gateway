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
            <h4 class="shell-heading mt-1 text-base font-semibold">
                Technical Details
            </h4>
        </div>
        <div class="flex h-8 w-10 items-end justify-center rounded-md border pb-1.5"
            style="border-color: var(--line-soft); background: var(--panel-soft);">
            <SignalStrengthIndicator rssi={simInfo?.signal_quality?.rssi || 99} compact={true} />
        </div>
    </div>

    <SignalStrengthIndicator rssi={simInfo?.signal_quality?.rssi || 99} />

    <SimCardEnhancedInfo {simInfo} />
</div>
