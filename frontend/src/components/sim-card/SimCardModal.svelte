<!-- frontend/src/lib/components/simcard/SimCardModal.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import { onMount } from "svelte";
    import { simCards, simCardActions } from "../../stores/simcards";
    import { apiClient } from "../../js/api.js";
    import Modal from "../common/Modal.svelte";
    import SimCardTabs from "./SimCardTabs.svelte";
    import SimCardTabContent from "./SimCardTabContent.svelte";

    // Props
    let { isOpen = false, onClose = () => {} } = $props();

    // State management
    let activeSimId = $state(null);
    let tabDataCache = $state({});
    let isCurrentTabLoading = $state(false);

    // Load SIM cards on mount
    onMount(async () => {
        await simCardActions.loadAll();
    });

    // Track modal open state and initialize
    let previousIsOpen = false;
    $effect(() => {
        if (isOpen && !previousIsOpen && $simCards.length > 0) {
            if (!activeSimId || !$simCards.find((s) => s.id === activeSimId)) {
                activeSimId = $simCards[0]?.id;
            }
            if (activeSimId) {
                loadTabData(activeSimId);
            }
            previousIsOpen = true;
        } else if (!isOpen) {
            previousIsOpen = false;
        }
    });

    // Load data for specific SIM tab
    /**
     * @param {string | number} simId
     */
    async function loadTabData(simId) {
        if (!simId) return;

        isCurrentTabLoading = true;

        try {
            const response = await apiClient.getSimInfo(simId);
            const simInfo = response.data || response;
            tabDataCache[simId] = simInfo;
            tabDataCache = { ...tabDataCache };
        } catch (error) {
            console.error(`Failed to load SIM info for ${simId}:`, error);
            tabDataCache[simId] = null;
            tabDataCache = { ...tabDataCache };
        } finally {
            isCurrentTabLoading = false;
        }
    }

    // Switch to different tab
    function switchTab(simId) {
        if (activeSimId === simId) return;

        activeSimId = simId;
        if (!tabDataCache[simId]) {
            loadTabData(simId);
        }
    }

    // Refresh current tab data
    function refreshCurrentTab() {
        if (activeSimId) {
            loadTabData(activeSimId);
        }
    }

    // Update handlers
    async function handleUpdatePhone(simId, phone) {
        const success = await simCardActions.updatePhoneNumber(simId, phone);
        if (success) {
            // Refresh cache if needed
            if (tabDataCache[simId]) {
                await loadTabData(simId);
            }
        }
        return success;
    }

    async function handleUpdateAlias(simId, alias) {
        const success = await simCardActions.updateAlias(simId, alias);
        if (success) {
            // Refresh cache if needed
            if (tabDataCache[simId]) {
                await loadTabData(simId);
            }
        }
        return success;
    }

    // Helper functions
    function getSimDisplayName(simCard) {
        return (
            simCard.alias ||
            simCard.phone_number ||
            `SIM ${simCard.id.slice(-8)}`
        );
    }

    // Derived state
    const activeSimCard = $derived(
        $simCards.find((sim) => sim.id === activeSimId),
    );
    const activeSimInfo = $derived(tabDataCache[activeSimId]);
</script>

<Modal {isOpen} {onClose}>
    <div
        class="flex flex-col gap-3 border-b px-5 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-6"
        style="border-color: var(--line-soft);"
    >
        <div class="flex min-w-0 items-center gap-3">
            <div class="shell-icon-badge shrink-0">
                <Icon icon="carbon:sim-card" class="h-4 w-4" />
            </div>
            <div class="min-w-0">
                <p class="shell-label">Device Detail</p>
                <h2 class="shell-heading text-xl font-semibold leading-tight sm:text-[1.4rem] lg:text-[1.75rem]">
                    SIM Card Information
                </h2>
            </div>
        </div>

        <div class="flex w-full items-center justify-between gap-3 sm:w-auto sm:justify-end">
            <span class="shell-chip shell-chip-muted text-[11px] sm:text-xs">
                {$simCards.length} SIM{$simCards.length === 1 ? '' : 's'}
            </span>
            <button
                class="shell-button h-10 w-10 px-0"
                onclick={() => onClose()}
            >
                <Icon icon="carbon:close" class="h-5 w-5" />
            </button>
        </div>
    </div>

    <SimCardTabs
        simCards={$simCards}
        {activeSimId}
        onTabSwitch={switchTab}
        getDisplayName={getSimDisplayName}
    />

    <div class="shell-scrollbar flex-1 overflow-auto px-4 py-4 sm:px-6 sm:py-5">
        <SimCardTabContent
            simCard={activeSimCard}
            simInfo={activeSimInfo}
            isLoading={isCurrentTabLoading}
            onRefresh={refreshCurrentTab}
            onUpdatePhone={(phone) => handleUpdatePhone(activeSimId, phone)}
            onUpdateAlias={(alias) => handleUpdateAlias(activeSimId, alias)}
            getDisplayName={getSimDisplayName}
        />
    </div>
</Modal>
