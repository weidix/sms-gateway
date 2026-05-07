<!-- frontend/src/lib/components/simcard/SimCardTabContent.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import { fly, fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { onDestroy } from "svelte";
    import SimCardBasicInfo from "./SimCardBasicInfo.svelte";
    import SimCardHealthInfo from "./SimCardHealthInfo.svelte";
    import SimCardTechnicalInfo from "./SimCardTechnicalInfo.svelte";
    import SimCardBasicInfoSkeleton from "./SimCardBasicInfoSkeleton.svelte";
    import SimCardTechnicalInfoSkeleton from "./SimCardTechnicalInfoSkeleton.svelte";
    import EmptyState from "./EmptyState.svelte";
    import AtDebugModal from "./AtDebugModal.svelte";

    let {
        simCard = null,
        simInfo = null,
        isLoading = false,
        onRefresh = () => {},
        onUpdatePhone = async (phone) => true,
        onUpdateAlias = async (alias) => true,
        getDisplayName = (sim) =>
            sim.alias || sim.phone_number || `SIM ${sim.id.slice(-8)}`,
    } = $props();

    let isRefreshing = $state(false);
    let minLoadingTimer = null;
    let skeletonStartTime = $state(null);
    let showSkeleton = $state(false);
    let isAtDebugOpen = $state(false);

    function handleRefresh() {
        if (isRefreshing) return;
        
        isRefreshing = true;
        const startTime = Date.now();
        const minLoadingDuration = 1000; // 1 second minimum
        
        // Clear any existing timer
        if (minLoadingTimer) {
            clearTimeout(minLoadingTimer);
        }
        
        // Start the actual refresh
        const refreshPromise = onRefresh();
        
        // Ensure minimum loading time
        minLoadingTimer = setTimeout(() => {
            isRefreshing = false;
        }, minLoadingDuration);
        
        // If refresh completes after minimum time, stop immediately
        Promise.resolve(refreshPromise).then(() => {
            const elapsed = Date.now() - startTime;
            if (elapsed >= minLoadingDuration) {
                if (minLoadingTimer) {
                    clearTimeout(minLoadingTimer);
                }
                isRefreshing = false;
            }
        }).catch(() => {
            const elapsed = Date.now() - startTime;
            if (elapsed >= minLoadingDuration) {
                if (minLoadingTimer) {
                    clearTimeout(minLoadingTimer);
                }
                isRefreshing = false;
            }
        });
    }

    // Manage skeleton display with minimum 1 second duration
    $effect(() => {
        if (isLoading && !simInfo) {
            // Starting to load - show skeleton and record time
            showSkeleton = true;
            skeletonStartTime = Date.now();
        } else if (simInfo && skeletonStartTime) {
            // Data loaded - check if minimum time has passed
            const elapsedTime = Date.now() - skeletonStartTime;
            const minSkeletonDuration = 1000; // 1 second minimum
            
            if (elapsedTime < minSkeletonDuration) {
                // Need to wait more time
                setTimeout(() => {
                    showSkeleton = false;
                    skeletonStartTime = null;
                }, minSkeletonDuration - elapsedTime);
            } else {
                // Minimum time already passed
                showSkeleton = false;
                skeletonStartTime = null;
            }
        }
    });

    // Cleanup timers on component destroy
    onDestroy(() => {
        if (minLoadingTimer) {
            clearTimeout(minLoadingTimer);
        }
    });

    function openAtDebug() {
        isAtDebugOpen = true;
    }

    function closeAtDebug() {
        isAtDebugOpen = false;
    }
</script>

{#if simCard}
    <div
        in:fly={{ x: 20, duration: 250, easing: cubicOut }}
        out:fade={{ duration: 150 }}
    >
        <div class="shell-card-compact mb-6 p-5 sm:p-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div class="flex items-center gap-4">
                    <div class="shell-icon-badge h-12 w-12 rounded-2xl">
                        <Icon
                            icon="carbon:sim-card"
                            class="h-5 w-5"
                        />
                    </div>
                    <div>
                        <p class="shell-label">Active Device</p>
                        <h3
                            class="shell-heading mt-1 text-2xl font-semibold"
                        >
                            {getDisplayName(simCard)}
                        </h3>
                        <p class="mt-2 text-sm font-mono" style="color: var(--text-muted);">
                            IMSI: {simCard.imsi}
                        </p>
                    </div>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    <button
                        class="shell-button"
                        onclick={openAtDebug}
                        title="Open AT debug"
                    >
                        <Icon icon="carbon:terminal" class="h-4 w-4" />
                        <span>AT Debug</span>
                    </button>
                    <button
                        class={`shell-button h-11 w-11 px-0 ${isRefreshing ? 'cursor-not-allowed opacity-75' : ''}`}
                        onclick={handleRefresh}
                        title="Refresh SIM info"
                        disabled={isRefreshing}
                    >
                        <Icon
                            icon="carbon:restart"
                            class={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`}
                        />
                    </button>
                    {#if simInfo?.model_info?.model}
                        <span class="shell-chip">
                            {simInfo.model_info.model}
                        </span>
                    {/if}
                </div>
            </div>
        </div>
        
        <div class="overflow-y-auto max-h-[60vh] sm:max-h-none">
            {#if showSkeleton || (isLoading && !simInfo) || isRefreshing}
                <div
                    class="space-y-4 lg:space-y-6 min-h-[18rem]"
                    in:fade={{ duration: 200 }}
                >
                    <div class="w-full">
                        <div class="shell-card-muted space-y-4 p-4 sm:p-5">
                            <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                                <div class="flex items-center gap-2">
                                    <div class="h-4 w-4 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-5 w-16 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                                <div class="h-6 w-24 animate-pulse rounded-full bg-black/10 dark:bg-white/10"></div>
                            </div>

                            <div class="grid gap-4 border-t pt-4 sm:grid-cols-2 lg:grid-cols-3"
                                style="border-color: var(--line-soft);"
                            >
                                <div class="space-y-2">
                                    <div class="h-3 w-24 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-4 w-20 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                                <div class="space-y-2">
                                    <div class="h-3 w-28 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-4 w-10 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                                <div class="space-y-2 sm:col-span-2 lg:col-span-3">
                                    <div class="h-3 w-24 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="flex flex-wrap gap-2">
                                        <div class="h-6 w-24 animate-pulse rounded-full bg-black/10 dark:bg-white/10"></div>
                                        <div class="h-6 w-28 animate-pulse rounded-full bg-black/10 dark:bg-white/10"></div>
                                    </div>
                                </div>
                                <div class="space-y-2">
                                    <div class="h-3 w-32 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-4 w-36 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                                <div class="space-y-2">
                                    <div class="h-3 w-32 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-4 w-28 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                                <div class="space-y-2">
                                    <div class="h-3 w-28 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                    <div class="h-4 w-36 animate-pulse rounded bg-black/10 dark:bg-white/10"></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
                        <SimCardBasicInfoSkeleton />
                        <SimCardTechnicalInfoSkeleton />
                    </div>
                </div>
            {:else if simInfo}
                <div
                    class="space-y-4 lg:space-y-6"
                    in:fade={{ delay: 100, duration: 300 }}
                >
                    <SimCardHealthInfo {simInfo} />

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
                        <SimCardBasicInfo
                            {simCard}
                            {simInfo}
                            {onUpdatePhone}
                            {onUpdateAlias}
                        />
                        <SimCardTechnicalInfo {simInfo} />
                    </div>
                </div>
            {:else}
                <EmptyState
                    title="No Data Available"
                    description="Unable to load information for this SIM card."
                    showRetry={true}
                    onRetry={onRefresh}
                />
            {/if}
        </div>

        <AtDebugModal
            isOpen={isAtDebugOpen}
            simId={simCard.id}
            simLabel={getDisplayName(simCard)}
            portLabel={simInfo?.com_port}
            onClose={closeAtDebug}
        />
    </div>
{:else}
    <EmptyState
        title="No SIM Cards Found"
        description="No SIM cards are currently available."
    />
{/if}
