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
    <div class="flex items-center justify-between gap-4 border-b p-4 sm:p-5"
        style="border-color: var(--line-soft);"
    >
        <div class="flex items-center gap-3 min-w-0">
            <div class="shell-icon-badge h-10 w-10 rounded-xl shrink-0">
                <span class="text-xs font-semibold">AT</span>
            </div>
            <div class="min-w-0">
                <p class="shell-label">Raw Transport</p>
                <h3 class="shell-heading mt-1 text-2xl font-semibold">
                    AT Debug
                </h3>
                <p class="text-xs sm:text-sm truncate" style="color: var(--text-muted);">
                    {simLabel} · {portLabel || "Unknown port"} · single run output
                </p>
            </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
            <span class="shell-chip">
                Raw Command
            </span>
            <button
                class="shell-button h-11 w-11 px-0"
                onclick={closeModal}
                aria-label="Close AT debug"
            >
                <Icon icon="carbon:close" class="h-5 w-5" />
            </button>
        </div>
    </div>

    <div class="border-b p-4 sm:p-5" style="border-color: var(--line-soft); background: var(--panel-soft);">
        <div class="flex items-center gap-3">
            <input
                type="text"
                bind:value={command}
                onkeydown={handleKeydown}
                placeholder="Enter raw AT command"
                class="shell-input h-11 flex-1 px-4 font-mono"
            />
            <button
                class={`shell-button h-11 px-5 ${isRunning || !command.trim() ? 'cursor-not-allowed opacity-50' : 'shell-button-primary'}`}
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
                <div class="flex items-center gap-2 text-xs" style="color: var(--text-muted);">
                    <span class={`px-2.5 py-1 rounded-full font-semibold ${statusClasses(execution.status)}`}>
                        {formatStatus(execution.status)}
                    </span>
                    <span>{formatExecutionTime(execution.executed_at)}</span>
                    <span>{formatDuration(execution.duration_ms)}</span>
                </div>
            {/if}
        </div>

        <div
            class="shell-scrollbar flex-1 min-h-[16rem] rounded-[24px] border px-4 py-3 font-mono text-sm
                   whitespace-pre-wrap break-words overflow-auto leading-7"
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
