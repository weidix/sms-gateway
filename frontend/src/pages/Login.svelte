<script>
    import Icon from "@iconify/svelte";
    import { onMount } from "svelte";
    import { fade, fly } from 'svelte/transition'; 
    import { quintOut } from 'svelte/easing';
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

<div class="shell-page relative overflow-y-auto px-4 py-5 sm:px-6 sm:py-7 lg:px-10">
    <div class="mx-auto flex w-full max-w-7xl items-center justify-between">
        <div class="flex items-center gap-3">
            <div class="shell-icon-badge">
                <Icon icon="carbon:send-filled" class="h-5 w-5" />
            </div>
            <div>
                <p class="shell-label">Private Relay</p>
                <h1 class="shell-heading text-lg font-semibold">SMS Gateway</h1>
            </div>
        </div>
    </div>

    <div class="mx-auto flex min-h-[calc(100dvh-5.5rem)] w-full max-w-7xl items-center">
        <div class="grid w-full gap-6 lg:grid-cols-[minmax(0,0.92fr)_minmax(22rem,30rem)] lg:gap-10">
            <section class="hidden flex-col justify-center gap-6 pr-0 lg:flex lg:pr-8 xl:pr-10">
                <div class="space-y-4" in:fly={{ y: 20, duration: 420, easing: quintOut }}>
                    <p class="shell-label">Secure Messaging</p>
                    <h2 class="shell-heading max-w-xl text-4xl font-semibold leading-[0.98] xl:text-5xl">
                        Sign in quickly and get back to the message queue.
                    </h2>
                    <p class="max-w-lg text-base leading-7 shell-subtitle">
                        The compact view stays focused on authentication. Wider screens keep a small amount of product context without pushing the form out of frame.
                    </p>
                </div>
            </section>

            <form
                class="relative flex w-full items-center justify-center lg:justify-end"
                onsubmit={handleSubmit}
                autocomplete="off"
                in:fly={{ y: 24, duration: 420, easing: quintOut }}
            >
                <div class="shell-card w-full max-w-md overflow-hidden p-5 sm:p-7 lg:p-8">
                    <div class="mb-6 flex items-start justify-between gap-4">
                        <div class="space-y-2">
                            <p class="shell-label">Secure Access</p>
                            <div>
                                <h3 class="shell-heading text-3xl font-semibold">Sign in</h3>
                                <p class="mt-2 text-sm leading-6 shell-subtitle">
                                    Authenticate to continue into the messaging workspace.
                                </p>
                            </div>
                        </div>
                        <div class="shell-icon-badge">
                            <Icon icon="carbon:user-avatar-filled" class="h-6 w-6" />
                        </div>
                    </div>

                    {#if error}
                        <div
                            class="mb-6 rounded-[22px] border px-4 py-4"
                            style="border-color: rgba(163, 83, 75, 0.24); background: rgba(163, 83, 75, 0.1);"
                            role="alert"
                            in:fade={{ duration: 200 }}
                        >
                            <div class="flex items-start gap-3">
                                <Icon icon="carbon:warning-filled" class="mt-0.5 h-5 w-5 shrink-0" style="color: var(--danger-strong);" />
                                <p class="text-sm font-medium" style="color: var(--danger-strong);">{error}</p>
                            </div>
                        </div>
                    {/if}

                    <div class="space-y-4">
                        <div>
                            <label for="username" class="mb-2 block text-sm font-medium" style="color: var(--text-secondary);">
                                Username
                            </label>
                            <div class="relative">
                                <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4" style="color: var(--text-muted);">
                                    <Icon icon="carbon:user" class="h-5 w-5" />
                                </div>
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
                                    class="shell-input pl-11"
                                    disabled={isLoading}
                                    required
                                />
                            </div>
                        </div>

                        <div>
                            <label for="password" class="mb-2 block text-sm font-medium" style="color: var(--text-secondary);">
                                Password
                            </label>
                            <div class="relative">
                                <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4" style="color: var(--text-muted);">
                                    <Icon icon="carbon:locked" class="h-5 w-5" />
                                </div>
                                <input
                                    id="password"
                                    name="login-password"
                                    type={showPassword ? "text" : "password"}
                                    bind:value={password}
                                    placeholder="Enter your password"
                                    autocomplete="new-password"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    class="shell-input pl-11 pr-12"
                                    disabled={isLoading}
                                    required
                                />
                                <button
                                    type="button"
                                    onclick={togglePassword}
                                    class="absolute inset-y-0 right-0 flex items-center pr-4"
                                    tabindex="-1"
                                    style="color: var(--text-muted);"
                                >
                                    <Icon
                                        icon={showPassword ? "carbon:view-off" : "carbon:view"}
                                        class="h-5 w-5 transition-colors duration-200 hover:opacity-70"
                                    />
                                </button>
                            </div>
                        </div>

                        <div class="flex flex-col items-start gap-3 pt-1 sm:flex-row sm:items-center sm:justify-between">
                            <label class="flex cursor-pointer items-center gap-2.5 select-none text-sm" style="color: var(--text-secondary);">
                                <input
                                    type="checkbox"
                                    bind:checked={rememberMe}
                                    class="h-4 w-4 rounded border-0"
                                    style="accent-color: var(--accent-copper);"
                                />
                                <span>Remember me</span>
                            </label>

                            <span class="shell-chip shell-chip-muted">
                                Local-only session
                            </span>
                        </div>
                    </div>

                    <button
                        type="submit"
                        class={`shell-button shell-button-primary mt-6 w-full ${isLoading || !username || !password ? 'cursor-not-allowed opacity-50' : ''}`}
                        disabled={isLoading || !username || !password}
                    >
                        {#if isLoading}
                            <Icon icon="carbon:circle-dash" class="h-5 w-5 animate-spin" />
                            <span>Signing in...</span>
                        {:else}
                            <Icon icon="carbon:login" class="h-5 w-5" />
                            <span>Sign in</span>
                        {/if}
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>
