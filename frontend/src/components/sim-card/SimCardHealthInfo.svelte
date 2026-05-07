<script>
    import Icon from "@iconify/svelte";

    let { simInfo = null } = $props();

    const statusDisplay = {
        healthy: {
            label: "Healthy",
            badgeClass: "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300"
        },
        degraded: {
            label: "Degraded",
            badgeClass: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-300"
        },
        recovering: {
            label: "Recovering",
            badgeClass: "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300"
        },
        critical: {
            label: "Critical",
            badgeClass: "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300"
        },
        unknown: {
            label: "Unknown",
            badgeClass: "bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300"
        }
    };

    function formatLabel(value) {
        if (!value) {
            return "Unavailable";
        }

        return value
            .split("_")
            .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
            .join(" ");
    }

    function formatTimestamp(value) {
        if (!value) {
            return "Unavailable";
        }

        const date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return value;
        }

        return new Intl.DateTimeFormat(undefined, {
            year: "numeric",
            month: "short",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit"
        }).format(date);
    }

    const healthStatus = $derived(simInfo?.health_status ?? null);
    const failureReasons = $derived(Array.isArray(simInfo?.failure_reasons) ? simInfo.failure_reasons : null);
    const consecutiveFailures = $derived(simInfo?.consecutive_failures ?? null);
    const lastSuccessfulProbe = $derived(simInfo?.last_ok_at ?? null);
    const lastRecoveryAction = $derived(simInfo?.last_recovery_action ?? null);
    const lastRecoveryTime = $derived(simInfo?.last_recovery_at ?? null);
    const hasAnyHealthData = $derived(
        healthStatus !== null ||
        failureReasons !== null ||
        consecutiveFailures !== null ||
        lastSuccessfulProbe !== null ||
        lastRecoveryAction !== null ||
        lastRecoveryTime !== null
    );
    const healthBadge = $derived.by(() => {
        if (healthStatus === null) {
            return null;
        }

        const knownStatus = statusDisplay[healthStatus];
        if (knownStatus) {
            return knownStatus;
        }

        return {
            label: formatLabel(healthStatus),
            badgeClass: statusDisplay.unknown.badgeClass
        };
    });
</script>

{#if hasAnyHealthData}
    <div class="shell-data-card space-y-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div class="flex items-center gap-2">
                <Icon icon="mage:activity" class="h-4 w-4" style="color: var(--text-muted);" />
                <div>
                    <p class="shell-label">Health</p>
                    <h4 class="shell-heading mt-1 text-xl font-semibold">Operational Health</h4>
                </div>
            </div>

            {#if healthBadge}
                <span class={`inline-flex items-center self-start rounded-full px-2.5 py-1 text-xs font-semibold ${healthBadge.badgeClass}`}>
                    {healthBadge.label}
                </span>
            {/if}
        </div>

        <div class="grid gap-4 border-t pt-4 sm:grid-cols-2 lg:grid-cols-3"
            style="border-color: var(--line-soft);"
        >
            <div class="text-sm">
                <div class="shell-data-label">Current Status</div>
                <div class="mt-1">
                    {#if healthBadge}
                        <span class="font-medium" style="color: var(--text-strong);">{healthBadge.label}</span>
                    {:else}
                        <span class="font-medium" style="color: var(--text-secondary);">Unavailable</span>
                    {/if}
                </div>
            </div>

            <div class="text-sm">
                <div class="shell-data-label">Consecutive Failures</div>
                <div class="mt-1 font-medium" style="color: var(--text-strong);">
                    {consecutiveFailures ?? "Unavailable"}
                </div>
            </div>

            <div class="text-sm sm:col-span-2 lg:col-span-3">
                <div class="shell-data-label">Failure Reasons</div>
                <div class="mt-1">
                    {#if failureReasons === null}
                        <span class="font-medium" style="color: var(--text-secondary);">Unavailable</span>
                    {:else if failureReasons.length === 0}
                        <span class="font-medium" style="color: var(--text-secondary);">None</span>
                    {:else}
                        <div class="flex flex-wrap gap-2">
                            {#each failureReasons as reason}
                                <span class="shell-chip shell-chip-warning">
                                    {formatLabel(reason)}
                                </span>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>

            <div class="text-sm">
                <div class="shell-data-label">Last Successful Probe</div>
                <div class="mt-1 font-medium" style="color: var(--text-strong);">
                    {formatTimestamp(lastSuccessfulProbe)}
                </div>
            </div>

            <div class="text-sm">
                <div class="shell-data-label">Last Recovery Action</div>
                <div class="mt-1 font-medium" style="color: var(--text-strong);">
                    {formatLabel(lastRecoveryAction)}
                </div>
            </div>

            <div class="text-sm">
                <div class="shell-data-label">Last Recovery Time</div>
                <div class="mt-1 font-medium" style="color: var(--text-strong);">
                    {formatTimestamp(lastRecoveryTime)}
                </div>
            </div>
        </div>
    </div>
{/if}
