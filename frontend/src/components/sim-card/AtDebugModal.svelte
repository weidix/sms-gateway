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
    overlayClass="z-[60] bg-zinc-100/60 dark:bg-zinc-950/50"
>
    <div class="flex items-center justify-between gap-4 p-4 sm:p-5 border-b border-gray-200 dark:border-zinc-700">
        <div class="flex items-center gap-3 min-w-0">
            <div class="w-9 h-9 rounded-lg bg-gray-900 dark:bg-gray-100 flex items-center justify-center shrink-0">
                <span class="text-xs font-semibold text-gray-100 dark:text-gray-900">AT</span>
            </div>
            <div class="min-w-0">
                <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100">
                    AT Debug
                </h3>
                <p class="text-xs sm:text-sm text-gray-500 dark:text-gray-400 truncate">
                    {simLabel} · {portLabel || "Unknown port"} · single run output
                </p>
            </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
            <span class="px-3 py-1 rounded-full bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300 text-xs font-semibold">
                Raw Command
            </span>
            <button
                class="p-2 rounded-lg bg-gray-100 dark:bg-zinc-800 text-gray-600 dark:text-gray-400
                       transition-colors duration-200 hover:bg-gray-200 dark:hover:bg-zinc-700
                       hover:text-gray-800 dark:hover:text-gray-200"
                onclick={closeModal}
                aria-label="Close AT debug"
            >
                <Icon icon="carbon:close" class="w-5 h-5" />
            </button>
        </div>
    </div>

    <div class="p-4 sm:p-5 border-b border-gray-200 dark:border-zinc-700 bg-gray-50/80 dark:bg-zinc-900/40">
        <div class="flex items-center gap-3">
            <input
                type="text"
                bind:value={command}
                onkeydown={handleKeydown}
                placeholder="Enter raw AT command"
                class="flex-1 h-11 rounded-xl border border-gray-300 dark:border-zinc-600
                       bg-white dark:bg-zinc-800 px-4 text-sm text-gray-900 dark:text-gray-100
                       font-mono outline-none transition-colors duration-200
                       focus:border-gray-500 dark:focus:border-zinc-500"
            />
            <button
                class="h-11 px-5 rounded-xl bg-gray-900 dark:bg-gray-100 text-gray-100 dark:text-gray-900
                       font-semibold text-sm transition-all duration-200 hover:bg-gray-800 dark:hover:bg-gray-200
                       active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50"
                onclick={handleSubmit}
                disabled={isRunning || !command.trim()}
            >
                {isRunning ? "Sending..." : "Send"}
            </button>
        </div>
    </div>

    <div class="flex-1 min-h-0 p-4 sm:p-5 flex flex-col gap-3">
        <div class="flex items-center justify-between gap-3">
            <div class="text-sm text-gray-500 dark:text-gray-400">Current execution</div>
            {#if execution}
                <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                    <span class={`px-2.5 py-1 rounded-full font-semibold ${statusClasses(execution.status)}`}>
                        {formatStatus(execution.status)}
                    </span>
                    <span>{formatExecutionTime(execution.executed_at)}</span>
                    <span>{formatDuration(execution.duration_ms)}</span>
                </div>
            {/if}
        </div>

        <div
            class="flex-1 min-h-[16rem] rounded-2xl border border-gray-200 dark:border-zinc-700
                   bg-white dark:bg-zinc-900 px-4 py-3 font-mono text-sm text-gray-700 dark:text-gray-200
                   whitespace-pre-wrap break-words overflow-auto leading-7"
        >
            {#if isRunning}
                Executing...
            {:else if execution?.output}
                {execution.output}
            {/if}
        </div>
    </div>
</Modal>
