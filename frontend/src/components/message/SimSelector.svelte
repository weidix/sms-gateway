<script>
  import Icon from "@iconify/svelte";
  import { simCards } from "../../stores/simcards";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  let { selectedSim = $bindable(null) } = $props();

  let showSimSelector = $state(false);
  let searchText = $state("");
  let expandedContent = $state(false);

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
    expandedContent = false;
    setTimeout(() => {
      showSimSelector = false;
    }, 200);
    searchText = "";
  }

  function toggleSimSelector() {
    if (!showSimSelector) {
      showSimSelector = true;
      requestAnimationFrame(() => {
        expandedContent = true;
      });
    } else {
      expandedContent = false;
      setTimeout(() => {
        showSimSelector = false;
      }, 200);
    }
    searchText = "";
  }

  function clickOutside(node) {
    const handleClick = (event) => {
      if (!node.contains(event.target)) {
        expandedContent = false;
        setTimeout(() => {
          showSimSelector = false;
        }, 200);
        searchText = "";
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

<div class="relative" use:clickOutside>
  <div
    class="sim-selector-width relative transition-all duration-300 ease-out w-full sm:max-w-none"
    style={`--sim-width: ${showSimSelector ? '320px' : '200px'};`}
  >
    {#if expandedContent}
      <!-- 向上弹出的内容 -->
      <div
        class="absolute bottom-full left-0 right-0 overflow-y-hide scrollbar-hide"
        transition:fly={{ y: 100, duration: 200, easing: quintOut }}
      >
        <div
          class="max-h-[70vh] overflow-hidden rounded-t-[26px] border border-b-0 shadow-[var(--shadow-strong)]"
          style="border-color: var(--line-soft); background: var(--panel-strong);"
        >
          <div
            class="border-b px-4 py-3"
            style="border-color: var(--line-soft); background: var(--panel-soft);"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="shell-icon-badge h-5 w-5 rounded-md">
                  <Icon
                    icon="carbon:sim-card"
                    class="h-3 w-3"
                  />
                </div>
                <span
                  class="shell-label"
                >
                  Select SIM Card
                </span>
              </div>
              <button
                onclick={toggleSimSelector}
                class="flex h-8 w-8 items-center justify-center rounded-full transition-colors duration-200 hover:bg-black/5 dark:hover:bg-white/5"
              >
                <Icon
                  icon="carbon:close"
                  class="h-3.5 w-3.5"
                  style="color: var(--text-muted);"
                />
              </button>
            </div>
          </div>

          <div
            class="border-b px-3 py-3"
            style="border-color: var(--line-soft); background: rgba(255,255,255,0.04);"
          >
            <div class="relative">
              <Icon
                icon="carbon:search"
                class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2"
                style="color: var(--text-muted);"
              />
              <input
                type="text"
                bind:value={searchText}
                placeholder="Search by name or number..."
                class="shell-input h-11 pl-9 pr-3"
              />
            </div>
          </div>

          <div class="shell-scrollbar max-h-56 overflow-y-auto sm:max-h-64">
            <div class="py-1">
              {#each filteredSimCards as sim (sim.id)}
                {@const isSelected = sim.id === selectedSim?.id}
                <button
                  onclick={() => selectSim(sim)}
                  class={`flex w-full items-center gap-3 px-3 py-3 text-left transition-all duration-150 ${
                    isSelected
                      ? 'bg-[var(--accent-soft)]'
                      : 'hover:bg-black/5 dark:hover:bg-white/5'
                  }`}
                >
                  <div
                    class={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl transition-colors ${
                      isSelected
                        ? 'bg-[var(--panel-contrast)] text-[var(--paper-strong)]'
                        : 'border border-[color:var(--line-soft)] bg-[var(--panel-soft)] text-[var(--text-secondary)]'
                    }`}
                  >
                    <Icon
                      icon={isSelected ? "carbon:checkmark-filled" : "carbon:sim-card"}
                      class="h-4 w-4"
                    />
                  </div>
                  <div class="flex-1 text-left">
                    <div
                      class="text-sm font-medium"
                      style={`color: ${isSelected ? 'var(--text-strong)' : 'var(--text-secondary)'}`}
                    >
                      {sim.alias ||
                        sim.phone_number ||
                        `SIM ${sim.id.slice(-6)}`}
                    </div>
                    {#if sim.alias && sim.phone_number}
                      <div
                        class="text-xs font-mono"
                        style={`color: ${isSelected ? 'var(--text-secondary)' : 'var(--text-muted)'}`}
                      >
                        {sim.phone_number}
                      </div>
                    {/if}
                  </div>
                  {#if isSelected}
                    <div class="flex items-center gap-1.5">
                      <span class="shell-status-dot"></span>
                      <span
                        class="text-xs font-medium"
                        style="color: var(--text-muted);"
                      >
                        Active
                      </span>
                    </div>
                  {/if}
                </button>
              {:else}
                <div
                  class="py-8 text-center text-sm"
                  style="color: var(--text-muted);"
                >
                  <Icon
                    icon="carbon:search-locate-mirror"
                    class="mx-auto mb-2 h-8 w-8 opacity-50"
                  />
                  No SIM cards found
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- 主按钮 - 始终显示，根据状态改变样式 -->
    <div
      class={`relative border transition-all duration-250 ease-out ${
        expandedContent
          ? 'rounded-b-[26px] border-t-0 shadow-[var(--shadow-strong)]'
          : 'rounded-[24px]'
      }`}
      style="border-color: var(--line-soft); background: var(--panel-strong);"
    >
      {#if expandedContent}
        <div
          class="absolute left-0 right-0 top-0 h-1 -mt-[1px]"
          style="background: var(--panel-strong);"
        ></div>
      {/if}

      <button
        onclick={toggleSimSelector}
        class={`flex h-14 w-full items-center gap-3 px-4 transition-colors duration-200 ${
          expandedContent
            ? 'rounded-b-[26px]'
            : 'rounded-[24px] hover:bg-black/5 dark:hover:bg-white/5'
        }`}
      >
        <div class="shell-icon-badge h-9 w-9 rounded-xl flex-shrink-0">
          <Icon
            icon="carbon:sim-card"
            class="h-3.5 w-3.5"
          />
        </div>
        {#if selectedSim}
          <span
            class="flex-1 truncate text-left text-sm font-medium"
            style="color: var(--text-strong);"
          >
            {selectedSim.alias ||
              selectedSim.phone_number ||
              `SIM ${selectedSim.id.slice(-6)}`}
          </span>
          <div class="flex items-center gap-1.5">
            <span class="shell-status-dot"></span>
          </div>
        {:else}
          <span
            class="flex-1 text-left text-sm"
            style="color: var(--text-muted);"
          >
            Select SIM
          </span>
        {/if}
        <Icon
          icon="carbon:chevron-down"
          class="h-4 w-4 flex-shrink-0 transition-transform duration-300"
          style="transform: rotate({expandedContent ? '180deg' : '0deg'}); color: var(--text-muted);"
        />
      </button>
    </div>
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

  .sim-selector-width {
    width: min(var(--sim-width), 100%);
  }

  @media (min-width: 640px) {
    .sim-selector-width {
      width: var(--sim-width);
    }
  }
</style>
