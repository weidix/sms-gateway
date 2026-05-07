<!-- frontend/src/lib/components/simcard/SimCardBasicInfo.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import EditableField from "../common/EditableField.svelte";
    
    let {
        simCard = {},
        simInfo = null,
        onUpdatePhone = async (phone) => true,
        onUpdateAlias = async (alias) => true
    } = $props();
    
    function getStatusDescription(status) {
        const statusMap = {
            "0": "Not registered",
            "1": "Registered (Home)",
            "2": "Searching",
            "3": "Registration denied", 
            "5": "Registered (Roaming)"
        };
        return statusMap[status] || `Status ${status}`;
    }
</script>

<div class="shell-data-card space-y-4">
    <div class="flex items-center justify-between">
        <div>
            <p class="shell-label">Profile</p>
            <h4 class="shell-heading mt-1 text-lg font-semibold sm:text-xl">
                Basic Information
            </h4>
        </div>
        <div class="shell-icon-badge-muted h-10 w-10 rounded-xl">
            <Icon icon="mage:phone" class="h-4 w-4" />
        </div>
    </div>

    <EditableField
        value={simCard.phone_number}
        icon="mage:phone"
        label="Phone Number"
        placeholder="Not set"
        onSave={onUpdatePhone}
    />

    <EditableField
        value={simCard.alias}
        icon="mage:tag"
        label="Alias"
        placeholder="Not set"
        onSave={onUpdateAlias}
    />

    <div class="shell-data-row rounded-[20px] border px-4 py-3"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        <div class="shell-icon-badge-muted h-10 w-10 rounded-xl">
            <Icon icon="mage:globe" class="h-4 w-4" />
        </div>
        <div>
            <div class="shell-data-label">Network Status</div>
            <div class="mt-1 text-sm font-medium" style="color: var(--text-strong);">
                {simInfo?.operator_info?.registration_status ? getStatusDescription(simInfo.operator_info.registration_status) : 'Unknown'}
            </div>
        </div>
    </div>

    <div class="shell-data-row rounded-[20px] border px-4 py-3"
        style="border-color: var(--line-soft); background: var(--panel-soft);"
    >
        <div class="shell-icon-badge-muted h-10 w-10 rounded-xl">
            <Icon icon="mage:building-b" class="h-4 w-4" />
        </div>
        <div>
            <div class="shell-data-label">Operator</div>
            <div class="mt-1 text-sm font-medium" style="color: var(--text-strong);">
                {simInfo?.operator_info?.operator_name || 'Unknown'}
            </div>
        </div>
    </div>
</div>
