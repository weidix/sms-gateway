<script>
    import Icon from "@iconify/svelte";
    import Modal from "../common/Modal.svelte";
    import { apiClient } from "../../js/api.js";

    let {
        isOpen = false,
        simId = "",
        simLabel = "",
        portLabel = "",
        onClose = () => {},
    } = $props();

    let command = $state("");
    let isRunning = $state(false);
    let execution = $state(null);
    let previousIsOpen = false;

    function resetState() {
        command = "";
        isRunning = false;
        execution = null;
    }

    $effect(() => {
        if (isOpen && !previousIsOpen) {
            resetState();
            previousIsOpen = true;
        } else if (!isOpen) {
            previousIsOpen = false;
        }
    });

    function closeModal() {
        resetState();
        onClose();
    }

    function formatStatus(status) {
        switch (status) {
            case "ok":
                return "OK";
            case "error":
                return "ERROR";
            case "transport_error":
                return "FAILED";
            default:
                return "UNKNOWN";
        }
    }

    function statusClasses(status) {
        switch (status) {
            case "ok":
                return "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300";
            case "error":
            case "transport_error":
                return "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300";
            default:
                return "bg-gray-200 text-gray-700 dark:bg-zinc-700 dark:text-gray-300";
        }
    }

    function formatExecutionTime(value) {
        if (!value) return "";

        const date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return value;
        }

        return new Intl.DateTimeFormat(undefined, {
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
        }).format(date);
    }

    function formatDuration(value) {
        if (typeof value !== "number" || Number.isNaN(value)) {
            return "";
        }

        if (value < 1000) {
            return `${value}ms`;
        }

        return `${(value / 1000).toFixed(2)}s`;
    }

    async function handleSubmit() {
        const trimmedCommand = command.trim();
        if (!trimmedCommand || isRunning || !simId) {
            return;
        }

        execution = null;
        isRunning = true;
        const startedAt = Date.now();

        try {
            const response = await apiClient.executeAtCommand(simId, trimmedCommand);
            execution = response.data || response;
        } catch (error) {
            const data = error?.data || {};
            execution = {
                command: trimmedCommand,
                output: data.output || data.error || "Failed to execute AT command.",
                status: data.status || "transport_error",
                executed_at: data.executed_at || new Date().toISOString(),
                duration_ms: data.duration_ms ?? Date.now() - startedAt,
            };
        } finally {
            isRunning = false;
        }
    }

    function handleKeydown(event) {
        if (event.key === "Enter") {
            event.preventDefault();
            handleSubmit();
        }
    }
</script>

<Modal
    {isOpen}
    onClose={closeModal}
    maxWidth="max-w-3xl"
    overlayClass="z-[60]"
>
    <div class="flex items-center justify-between gap-4 border-b px-4 py-3"
        style="border-color: var(--line-soft); background: var(--panel-strong);"
    >
        <div class="flex min-w-0 items-center gap-2.5">
            <div class="shell-icon-badge h-7 w-7 shrink-0 rounded">
                <span class="shell-mono text-[11px] font-semibold">AT</span>
            </div>
            <div class="min-w-0">
                <h3 class="shell-heading text-base font-semibold leading-tight">AT Debug</h3>
                <p class="shell-mono mt-0.5 truncate text-xs" style="color: var(--text-muted);">
                    {simLabel} · {portLabel || "Unknown port"}
                </p>
            </div>
        </div>

        <button
            class="shell-button h-8 w-8 px-0"
            onclick={closeModal}
            aria-label="Close AT debug"
        >
            <Icon icon="carbon:close" class="h-4 w-4" />
        </button>
    </div>

    <div class="border-b px-4 py-3" style="border-color: var(--line-soft); background: var(--panel-strong);">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
            <input
                type="text"
                bind:value={command}
                onkeydown={handleKeydown}
                placeholder="Enter raw AT command"
                class="shell-input shell-mono flex-1"
            />
            <button
                class={`shell-button w-full px-4 sm:w-auto ${isRunning || !command.trim() ? 'cursor-not-allowed opacity-50' : 'shell-button-primary'}`}
                onclick={handleSubmit}
                disabled={isRunning || !command.trim()}
            >
                {isRunning ? "Sending..." : "Send"}
            </button>
        </div>
    </div>

    <div class="flex-1 min-h-0 p-4 sm:p-5 flex flex-col gap-3">
        <div class="flex items-center justify-between gap-3">
            <div class="text-sm" style="color: var(--text-muted);">Current execution</div>
            {#if execution}
                <div class="shell-mono flex items-center gap-2 text-xs" style="color: var(--text-muted);">
                    <span class={`rounded-sm border px-1.5 py-0.5 font-semibold ${statusClasses(execution.status)}`}>
                        {formatStatus(execution.status)}
                    </span>
                    <span>{formatExecutionTime(execution.executed_at)}</span>
                    <span>{formatDuration(execution.duration_ms)}</span>
                </div>
            {/if}
        </div>

        <div
            class="shell-scrollbar min-h-[14rem] flex-1 rounded-md border px-3 py-2.5 font-mono text-[0.8125rem]
                   whitespace-pre-wrap break-words overflow-auto leading-6"
            style="border-color: var(--line-soft); background: var(--panel-soft); color: var(--text-secondary);"
        >
            {#if isRunning}
                Executing...
            {:else if execution?.output}
                {execution.output}
            {/if}
        </div>
    </div>
</Modal>
