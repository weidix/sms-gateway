<script>
    import Icon from "@iconify/svelte";
    import { onMount } from "svelte";
    import { fade } from 'svelte/transition';
    import { updateStorageValue } from '../js/storage';

    let username = "";
    let password = "";
    let error = "";
    let isLoading = false;
    let showPassword = false;
    let rememberMe = false;

    onMount(() => {
        const rememberedUsername = localStorage.getItem("auth_username");
        if (rememberedUsername) {
            username = rememberedUsername;
            rememberMe = true;
        }
    });

    const handleSubmit = (/** @type {{ preventDefault: () => void; }} */ e) => {
        e.preventDefault();
        handleLogin();
    };

    const handleLogin = async () => {
        if (!username || !password) {
            error = "Username and password are required";
            return;
        }

        try {
            isLoading = true;
            error = "";

            const authToken = btoa(`${username}:${password}`);

            const response = await fetch("/api/check", {
                method: "GET",
                headers: {
                    Authorization: `Basic ${authToken}`,
                    "Content-Type": "application/json",
                },
            });

            switch (response.status) {
                case 204:
                    await updateStorageValue("auth", {
                        username,
                        token: authToken,
                    });
                    rememberMe
                        ? localStorage.setItem("auth_username", username)
                        : localStorage.removeItem("auth_username");
                    window.location.reload();
                    break;

                case 401:
                    error = "Invalid credentials";
                    break;

                default:
                    error = `Unexpected error: HTTP ${response.status}`;
                    break;
            }
        } catch (err) {
            error = err.message.includes("Failed to fetch")
                ? "Unable to connect to the server"
                : "Authentication process failed";
        } finally {
            isLoading = false;
        }
    };

    function togglePassword() {
        showPassword = !showPassword;
    }
</script>

<div class="shell-page overflow-y-auto bg-[var(--bg-canvas)] px-4 py-6 sm:px-6 lg:px-10">
    <div class="mx-auto flex w-full max-w-7xl items-center justify-between">
        <div class="flex items-center gap-2.5">
            <div class="shell-icon-badge h-7 w-7 rounded">
                <Icon icon="carbon:send-filled" class="h-3.5 w-3.5" />
            </div>
            <div>
                <h1 class="shell-heading text-sm font-semibold leading-tight">SMS Gateway</h1>
                <p class="shell-label mt-0.5 normal-case tracking-normal">Private relay</p>
            </div>
        </div>
    </div>

    <div class="mx-auto flex min-h-[calc(100dvh-5.5rem)] w-full max-w-7xl items-center">
        <div class="grid w-full gap-6 lg:grid-cols-[minmax(0,0.92fr)_minmax(22rem,26rem)] lg:gap-10">
            <section class="hidden flex-col justify-center gap-4 pr-0 lg:flex lg:pr-8 xl:pr-10">
                <div class="space-y-3">
                    <p class="shell-label">Secure Messaging</p>
                    <h2 class="shell-heading max-w-md text-2xl font-semibold leading-tight xl:text-3xl">
                        Sign in to reach the message queue.
                    </h2>
                    <p class="max-w-sm text-sm leading-6 shell-subtitle">
                        Conversations, SIM health, and dispatch — all behind your own modem hardware.
                    </p>
                </div>
            </section>

            <form
                class="flex w-full items-center justify-center lg:justify-end"
                onsubmit={handleSubmit}
                autocomplete="off"
            >
                <div class="shell-card w-full max-w-sm p-5 sm:p-6">
                    <div class="mb-5">
                        <h3 class="shell-heading text-lg font-semibold">Sign in</h3>
                        <p class="mt-1 text-sm shell-subtitle">
                            Authenticate to continue.
                        </p>
                    </div>

                    {#if error}
                        <div
                            class="mb-4 rounded-md border px-3 py-2.5"
                            style="border-color: var(--danger-strong); background: transparent;"
                            role="alert"
                            in:fade={{ duration: 150 }}
                        >
                            <div class="flex items-start gap-2">
                                <Icon icon="carbon:warning-alt" class="mt-0.5 h-4 w-4 shrink-0" style="color: var(--danger-strong);" />
                                <p class="text-sm font-medium" style="color: var(--danger-strong);">{error}</p>
                            </div>
                        </div>
                    {/if}

                    <div class="space-y-3">
                        <div>
                            <label for="username" class="mb-1.5 block text-xs font-medium" style="color: var(--text-secondary);">
                                Username
                            </label>
                            <input
                                id="username"
                                name="login-username"
                                type="text"
                                bind:value={username}
                                placeholder="Enter your username"
                                autocomplete="off"
                                autocapitalize="none"
                                spellcheck="false"
                                inputmode="text"
                                aria-autocomplete="none"
                                class="shell-input h-10"
                                disabled={isLoading}
                                required
                            />
                        </div>

                        <div>
                            <label for="password" class="mb-1.5 block text-xs font-medium" style="color: var(--text-secondary);">
                                Password
                            </label>
                            <div class="relative">
                                <input
                                    id="password"
                                    name="login-password"
                                    type={showPassword ? "text" : "password"}
                                    bind:value={password}
                                    placeholder="Enter your password"
                                    autocomplete="new-password"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    class="shell-input h-10 pr-10"
                                    disabled={isLoading}
                                    required
                                />
                                <button
                                    type="button"
                                    onclick={togglePassword}
                                    class="absolute inset-y-0 right-0 flex items-center pr-3"
                                    tabindex="-1"
                                    style="color: var(--text-muted);"
                                >
                                    <Icon
                                        icon={showPassword ? "carbon:view-off" : "carbon:view"}
                                        class="h-4 w-4 transition-opacity duration-150 hover:opacity-70"
                                    />
                                </button>
                            </div>
                        </div>

                        <label class="flex cursor-pointer select-none items-center gap-2 pt-0.5 text-xs" style="color: var(--text-secondary);">
                            <input
                                type="checkbox"
                                bind:checked={rememberMe}
                                class="h-3.5 w-3.5 rounded border-0"
                                style="accent-color: var(--text-strong);"
                            />
                            <span>Remember me</span>
                        </label>
                    </div>

                    <button
                        type="submit"
                        class={`shell-button shell-button-primary mt-5 h-10 w-full ${isLoading || !username || !password ? 'cursor-not-allowed opacity-50' : ''}`}
                        disabled={isLoading || !username || !password}
                    >
                        {#if isLoading}
                            <Icon icon="carbon:circle-dash" class="h-4 w-4 animate-spin" />
                            <span>Signing in...</span>
                        {:else}
                            <Icon icon="carbon:login" class="h-4 w-4" />
                            <span>Sign in</span>
                        {/if}
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>
