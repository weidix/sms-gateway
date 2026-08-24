<!-- frontend/src/lib/components/common/EditableField.svelte -->
<script>
    import Icon from "@iconify/svelte";

    let {
        value = "",
        label = "Field",
        placeholder = "Not set",
        onSave = async (val) => true,
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

<div class="flex items-center justify-between gap-3 rounded-md border px-3 py-2"
    style="border-color: var(--line-soft); background: var(--panel-soft);"
>
    <div class="min-w-0">
        <div class="shell-data-label">{label}</div>
        <div class="mt-0.5 text-sm font-medium" style="color: var(--text-strong);">
            {#if isEditing}
                <input
                    type="text"
                    bind:value={tempValue}
                    bind:this={inputRef}
                    class="shell-input shell-mono h-8 w-full max-w-[12rem]"
                    onkeydown={handleKeydown}
                />
            {:else}
                <span class="shell-mono" class:italic={!value} style={!value ? 'color: var(--text-muted);' : ''}>
                    {value || placeholder}
                </span>
            {/if}
        </div>
    </div>
    <div class="flex shrink-0 items-center gap-1">
        {#if isEditing}
            <button
                class="flex h-7 w-7 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={save}
                aria-label="Save field"
            >
                <Icon icon="mage:check" width="14" height="14" style="color: var(--success-strong);" />
            </button>
            <button
                class="flex h-7 w-7 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={cancelEdit}
                aria-label="Cancel edit"
            >
                <Icon icon="mage:multiply" width="14" height="14" style="color: var(--text-muted);" />
            </button>
        {:else}
            <button
                class="flex h-7 w-7 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
                onclick={startEdit}
                aria-label="Edit field"
            >
                <Icon icon="mage:edit" width="14" height="14" style="color: var(--text-muted);" />
            </button>
        {/if}
    </div>
</div>
