<script>
  import Icon from "@iconify/svelte";
  import { simCards } from "../../stores/simcards";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  let { selectedSim = $bindable(null) } = $props();

  let showSimSelector = $state(false);
  let searchText = $state("");

  let filteredSimCards = $derived(
    $simCards.filter(
      (sim) =>
        !searchText ||
        (sim.alias &&
          sim.alias.toLowerCase().includes(searchText.toLowerCase())) ||
        (sim.phone_number && sim.phone_number.includes(searchText)) ||
        sim.id.toLowerCase().includes(searchText.toLowerCase()),
    ),
  );

  $effect(() => {
    const storedSimId = localStorage.getItem("selectedSimId");
    if (storedSimId && $simCards.length > 0) {
      const foundSim = $simCards.find((sim) => sim.id === storedSimId);
      selectedSim = foundSim || $simCards[0];
    } else if (!selectedSim && $simCards.length > 0) {
      selectedSim = $simCards[0];
    }
  });

  function selectSim(sim) {
    selectedSim = sim;
    localStorage.setItem("selectedSimId", sim.id);
    closeSimSelector();
  }

  function closeSimSelector() {
    showSimSelector = false;
    searchText = "";
  }

  function toggleSimSelector() {
    showSimSelector = !showSimSelector;
    if (!showSimSelector) {
      searchText = "";
    }
  }

  function clickOutside(node) {
    const handleClick = (event) => {
      if (!node.contains(event.target)) {
        closeSimSelector();
      }
    };
    document.addEventListener("click", handleClick, true);

    return {
      destroy() {
        document.removeEventListener("click", handleClick, true);
      },
    };
  }
</script>

<div class="sim-selector-shell relative" use:clickOutside>
  <div class="sim-selector-trigger w-full">
    {#if showSimSelector}
      <div
        class="absolute bottom-[calc(100%+0.75rem)] left-0 z-20 w-full max-w-[calc(100vw-1.5rem)] overflow-y-hide scrollbar-hide md:w-[var(--sim-panel-width)] md:max-w-none"
        transition:fly={{ y: 8, duration: 150, easing: quintOut }}
      >
        <div
          class="max-h-[70vh] overflow-hidden rounded-lg border shadow-[var(--shadow-strong)]"
          style="border-color: var(--line-strong); background: var(--panel-strong);"
        >
          <div
            class="flex items-center justify-between border-b px-3 py-2"
            style="border-color: var(--line-soft); background: var(--panel-soft);"
          >
            <span class="shell-label">Select SIM Card</span>
            <button
              onclick={closeSimSelector}
              class="flex h-6 w-6 items-center justify-center rounded transition-colors duration-150 hover:bg-black/5 dark:hover:bg-white/5"
              aria-label="Close SIM selector"
            >
              <Icon icon="carbon:close" class="h-3.5 w-3.5" style="color: var(--text-muted);" />
            </button>
          </div>

          <div
            class="border-b px-2.5 py-2"
            style="border-color: var(--line-soft);"
          >
            <div class="relative">
              <Icon
                icon="carbon:search"
                class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2"
                style="color: var(--text-muted);"
              />
              <input
                type="text"
                bind:value={searchText}
                placeholder="Search by name or number..."
                class="shell-input h-8 pl-8 pr-2 text-xs"
              />
            </div>
          </div>

          <div class="shell-scrollbar max-h-56 overflow-y-auto sm:max-h-64">
            <div class="p-1">
              {#each filteredSimCards as sim (sim.id)}
                {@const isSelected = sim.id === selectedSim?.id}
                <button
                  onclick={() => selectSim(sim)}
                  class={`flex w-full items-center gap-2.5 rounded-md px-2 py-2 text-left transition-colors duration-100 ${
                    isSelected
                      ? 'bg-[var(--panel-soft)]'
                      : 'hover:bg-black/5 dark:hover:bg-white/5'
                  }`}
                >
                  <Icon
                    icon={isSelected ? "carbon:checkmark" : "carbon:sim-card"}
                    class={`h-4 w-4 shrink-0 ${isSelected ? '' : ''}`}
                    style={`color: ${isSelected ? 'var(--text-strong)' : 'var(--text-muted)'};`}
                  />
                  <div class="min-w-0 flex-1">
                    <div
                      class="truncate text-sm font-medium"
                      style={`color: ${isSelected ? 'var(--text-strong)' : 'var(--text-secondary)'}`}
                    >
                      {sim.alias ||
                        sim.phone_number ||
                        `SIM ${sim.id.slice(-6)}`}
                    </div>
                    {#if sim.alias && sim.phone_number}
                      <div
                        class="shell-mono truncate text-xs"
                        style="color: var(--text-muted);"
                      >
                        {sim.phone_number}
                      </div>
                    {/if}
                  </div>
                  {#if isSelected}
                    <span class="shell-status-dot shrink-0"></span>
                  {/if}
                </button>
              {:else}
                <div
                  class="py-8 text-center text-sm"
                  style="color: var(--text-muted);"
                >
                  No SIM cards found
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}

    <button
      onclick={toggleSimSelector}
      class={`flex h-9 w-full items-center gap-2 rounded-md border px-2.5 text-left transition-colors duration-150 ${
        showSimSelector
          ? 'border-[color:var(--line-strong)]'
          : 'hover:border-[color:var(--line-strong)]'
      }`}
      style="border-color: var(--line-soft); background: var(--panel-strong);"
      aria-expanded={showSimSelector}
    >
      <Icon icon="carbon:sim-card" class="h-4 w-4 shrink-0" style="color: var(--text-muted);" />
      {#if selectedSim}
        <span
          class="min-w-0 flex-1 truncate text-sm font-medium"
          style="color: var(--text-strong);"
        >
          {selectedSim.alias ||
            selectedSim.phone_number ||
            `SIM ${selectedSim.id.slice(-6)}`}
        </span>
        <span class="shell-status-dot shrink-0"></span>
      {:else}
        <span class="min-w-0 flex-1 truncate text-sm" style="color: var(--text-muted);">
          Select SIM
        </span>
      {/if}
      <Icon
        icon="carbon:chevron-down"
        class="h-3.5 w-3.5 shrink-0 transition-transform duration-200"
        style="transform: rotate({showSimSelector ? '180deg' : '0deg'}); color: var(--text-muted);"
      />
    </button>
  </div>
</div>

<style lang="postcss">
  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }

  .scrollbar-hide {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }

  .sim-selector-shell {
    --sim-trigger-width: 220px;
    --sim-panel-width: 320px;
  }

  .sim-selector-trigger {
    width: min(var(--sim-trigger-width), 100%);
  }

  @media (min-width: 768px) {
    .sim-selector-trigger {
      width: var(--sim-trigger-width);
    }
  }
</style>
