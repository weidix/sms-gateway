<!-- frontend/src/lib/components/common/EditableField.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let {
        value = "",
        icon = "mage:tag",
        label = "Field",
        placeholder = "Not set",
        onSave = async (val) => true,
        inputClass = "w-32"
    } = $props();
    
    let isEditing = $state(false);
    let tempValue = $state("");
    let inputRef = $state();
    
    function startEdit() {
        isEditing = true;
        tempValue = value || "";
        // Focus input after DOM update
        setTimeout(() => {
            inputRef?.focus();
        }, 0);
    }
    
    function cancelEdit() {
        isEditing = false;
        tempValue = "";
    }
    
    async function save() {
        const success = await onSave(tempValue);
        if (success) {
            cancelEdit();
        }
    }
    
    async function handleKeydown(e) {
        if (e.key === 'Enter') await save();
        if (e.key === 'Escape') cancelEdit();
    }
</script>

<div class="flex items-start justify-between gap-4 rounded-[20px] border px-4 py-3"
    style="border-color: var(--line-soft); background: var(--panel-soft);"
>
    <div class="flex min-w-0 items-start gap-3">
        <div class="shell-icon-badge-muted h-10 w-10 rounded-xl">
            <Icon {icon} class="h-4 w-4" />
        </div>
        <div class="min-w-0">
            <div class="shell-data-label">{label}</div>
            <div class="mt-1 text-sm font-medium" style="color: var(--text-strong);">
                {#if isEditing}
                    <input
                        type="text"
                        bind:value={tempValue}
                        bind:this={inputRef}
                        class="shell-input h-10 {inputClass}"
                        onkeydown={handleKeydown}
                    />
                {:else}
                    {value || placeholder}
                {/if}
            </div>
        </div>
    </div>
    <div class="flex items-center gap-1">
        {#if isEditing}
            <button
                class="flex h-8 w-8 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={save}
                aria-label="Save field"
            >
                <Icon icon="mage:check" width="16" height="16" style="color: var(--success-strong);" />
            </button>
            <button
                class="flex h-8 w-8 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={cancelEdit}
                aria-label="Cancel edit"
            >
                <Icon icon="mage:multiply" width="16" height="16" style="color: var(--text-muted);" />
            </button>
        {:else}
            <button
                class="flex h-8 w-8 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={startEdit}
                aria-label="Edit field"
            >
                <Icon icon="mage:edit" width="16" height="16" style="color: var(--accent-copper);" />
            </button>
        {/if}
    </div>
</div>
