<script>

    let { simInfo = null } = $props();

    const statusDisplay = {
        healthy: {
            label: "Healthy",
            dotClass: "bg-green-600 dark:bg-green-500",
            textClass: "text-green-700 dark:text-green-400"
        },
        degraded: {
            label: "Degraded",
            dotClass: "bg-yellow-600 dark:bg-yellow-500",
            textClass: "text-yellow-700 dark:text-yellow-400"
        },
        recovering: {
            label: "Recovering",
            dotClass: "bg-amber-500 dark:bg-amber-400",
            textClass: "text-amber-700 dark:text-amber-400"
        },
        critical: {
            label: "Critical",
            dotClass: "bg-red-600 dark:bg-red-500",
            textClass: "text-red-700 dark:text-red-400"
        },
        unknown: {
            label: "Unknown",
            dotClass: "bg-gray-400 dark:bg-gray-500",
            textClass: "text-gray-600 dark:text-gray-400"
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
            dotClass: statusDisplay.unknown.dotClass,
            textClass: statusDisplay.unknown.textClass
        };
    });
</script>

{#if hasAnyHealthData}
    <div class="shell-data-card space-y-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
                <p class="shell-label">Health</p>
                <h4 class="shell-heading mt-1 text-base font-semibold">Operational Health</h4>
            </div>

            {#if healthBadge}
                <span class={`inline-flex items-center gap-1.5 self-start text-xs font-medium ${healthBadge.textClass}`}>
                    <span class={`h-1.5 w-1.5 rounded-full ${healthBadge.dotClass}`}></span>
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
