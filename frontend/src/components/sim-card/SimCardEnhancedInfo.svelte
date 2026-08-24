<!-- frontend/src/lib/components/simcard/SimCardEnhancedInfo.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let { simInfo = null } = $props();
    
    const hasEnhancedInfo = $derived(
        simInfo && (
            simInfo.sms_center || 
            simInfo.sim_status || 
            simInfo.memory_status || 
            simInfo.com_port || 
            simInfo.signal_quality || 
            simInfo.operator_info?.operator_id
        )
    );
</script>

{#if hasEnhancedInfo}
    <div class="space-y-3 border-t pt-4" style="border-color: var(--line-soft);">
        <h5 class="text-sm font-medium" style="color: var(--text-secondary);">Enhanced Information</h5>
        
        {#if simInfo.sms_center}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:message-dots" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">SMS Center:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">{simInfo.sms_center}</span>
            </div>
        {/if}
        
        {#if simInfo.sim_status}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:shield-check" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">SIM Status:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">{simInfo.sim_status}</span>
            </div>
        {/if}
        
        {#if simInfo.memory_status}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:memory-card" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">Memory:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">{simInfo.memory_status}</span>
            </div>
        {/if}
        
        {#if simInfo.com_port}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:link" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">Port:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">{simInfo.com_port} @ {simInfo.baud_rate}</span>
            </div>
        {/if}
        
        {#if simInfo.signal_quality}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:chart-up-b" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">Signal Detail:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">RSSI: {simInfo.signal_quality.rssi}, BER: {simInfo.signal_quality.ber}</span>
            </div>
        {/if}
        
        {#if simInfo.operator_info?.operator_id}
            <div class="shell-data-row text-sm">
                <Icon icon="mage:building-b" class="mt-0.5 h-4 w-4" style="color: var(--text-muted);" />
                <span style="color: var(--text-muted);">Operator ID:</span>
                <span class="shell-mono font-medium" style="color: var(--text-strong);">{simInfo.operator_info.operator_id}</span>
            </div>
        {/if}
    </div>
{/if}
