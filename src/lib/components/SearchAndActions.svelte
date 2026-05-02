<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    Filter,
    Users,
    UserCheck,
    Folder,
    Flag,
    CalendarDays,
    MessageSquareQuote,
    Settings,
    Rocket,
    SlidersHorizontal,
  } from "lucide-svelte";
  import ColumnSettingsDropdown from "./ColumnSettingsDropdown.svelte";

  export let isFiltersOpen: boolean = false;
  export let isOwner: boolean = false;
  export let isMyTasksActive: boolean = false;
  export let showTeam: boolean = true;
  export let showProjects: boolean = true;
  export let showSprints: boolean = true;
  export let showSummary: boolean = true;
  export let showDailyReflect: boolean = true;
  export let showWorkspaceSettings: boolean = true;
  export let showMilestones: boolean = true;

  let showColumnSettings = false;
  const dispatch = createEventDispatcher();

  let buttonEl: HTMLButtonElement;
  let filterButtonEl: HTMLButtonElement;
  let dropdownEl: HTMLDivElement;
  let dropdownPos = { top: 0, left: 0 };

  $: if (showColumnSettings && buttonEl) {
    const rect = buttonEl.getBoundingClientRect();
    dropdownPos = { top: rect.bottom + 8, left: rect.left };
  }

  function toggleColumnSettings() {
    const nextOpen = !showColumnSettings;
    if (nextOpen) dispatch("closeFilters");
    showColumnSettings = nextOpen;
  }

  function closeColumnSettings() {
    showColumnSettings = false;
  }

  function handleWindowClick(e: MouseEvent) {
    if (!showColumnSettings) return;
    const target = e.target as Node;
    if (buttonEl && buttonEl.contains(target)) return;
    if (dropdownEl && dropdownEl.contains(target)) return;
    closeColumnSettings();
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) node.parentNode.removeChild(node);
      },
    };
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") closeColumnSettings();
  }

  function toggleFilters() {
    closeColumnSettings();
    const rect = filterButtonEl?.getBoundingClientRect();
    dispatch("toggleFilters", rect ? { top: rect.bottom + 8, left: rect.left } : null);
  }

  function openWorkerManager() {
    dispatch("openWorkerManager");
  }

  function openProjectManager() {
    dispatch("openProjectManager");
  }

  function openSprintManager() {
    dispatch("openSprintManager");
  }

  function openMonthlySummary() {
    dispatch("openMonthlySummary");
  }

  function openDailyReflect() {
    dispatch("openDailyReflect");
  }

  function openMilestoneManager() {
    dispatch("openMilestoneManager");
  }

  function openWorkspaceSettings() {
    dispatch("openWorkspaceSettings");
  }

  function toggleMyTasks() {
    dispatch("toggleMyTasks");
  }

</script>

<svelte:window on:click={handleWindowClick} />

<div
  class="relative z-30 flex flex-col md:flex-row items-center gap-3 bg-white/50 dark:bg-gray-900/30 p-2 rounded-2xl border border-gray-200/50 dark:border-gray-800/50 backdrop-blur-sm transition-all group shadow-sm"
>
  <!-- Action Row (Search removed as requested) -->
  <div
    class="flex items-center gap-2.5 pb-1 md:pb-0 w-full md:w-auto overflow-x-auto scrollbar-hide"
  >
    <!-- Filter Toggle -->
    <button
      bind:this={filterButtonEl}
      on:click={toggleFilters}
      class="flex items-center justify-center w-12 h-12 shrink-0 border rounded-xl transition-all shadow-sm {isFiltersOpen
        ? 'bg-primary/10 border-primary text-primary'
        : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'}"
      title={$_("page__filters")}
    >
      <Filter size={20} />
    </button>

    <div class="relative">
      <button
        bind:this={buttonEl}
        on:click={toggleColumnSettings}
        class="flex items-center justify-center w-12 h-12 shrink-0 border rounded-xl transition-all shadow-sm {showColumnSettings
          ? 'bg-primary/10 border-primary text-primary'
          : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'}"
        title={$_("page__display_settings")}
      >
        <SlidersHorizontal size={20} />
      </button>

      {#if showColumnSettings}
        <div
          use:portal
          bind:this={dropdownEl}
          class="fixed z-[9999]"
          style="top: {dropdownPos.top}px; left: {dropdownPos.left}px;"
        >
          <ColumnSettingsDropdown on:close={closeColumnSettings} />
        </div>
      {/if}
    </div>

    {#if showTeam}
      <button
        on:click={openWorkerManager}
        class="flex items-center justify-center w-12 h-12 shrink-0 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-all shadow-sm"
        title={$_("page__team")}
      >
        <Users size={20} />
      </button>
    {/if}

    {#if isOwner && showWorkspaceSettings}
      <button
        on:click={openWorkspaceSettings}
        class="flex items-center justify-center w-12 h-12 shrink-0 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-all shadow-sm"
        title={$_("page__workspace_settings")}
      >
        <Settings size={20} />
      </button>
    {/if}

    <button
      on:click={toggleMyTasks}
      class="flex items-center justify-center w-12 h-12 shrink-0 border rounded-xl transition-all shadow-sm {isMyTasksActive
        ? 'bg-teal-500/15 border-teal-500/40 text-teal-300 dark:text-teal-300'
        : 'bg-teal-500/5 border-teal-500/15 text-teal-600 dark:text-teal-400 hover:bg-teal-500/10'}"
      title={$_("page__my_tasks")}
    >
      <UserCheck size={20} />
    </button>

    <div
      class="h-8 w-px bg-gray-200 dark:bg-gray-700 mx-1 hidden md:block"
    ></div>

    {#if showProjects}
      <button
        on:click={openProjectManager}
        class="flex items-center gap-2 px-4 h-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-400 font-bold text-[11px] uppercase tracking-widest transition-all shadow-sm hover:text-primary whitespace-nowrap"
        title={$_("page__projects")}
      >
        <Folder size={16} />
        <span class="hidden lg:inline">{$_("page__projects")}</span>
      </button>
    {/if}

    {#if showSprints}
      <button
        on:click={openSprintManager}
        class="flex items-center gap-2 px-4 h-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-400 font-bold text-[11px] uppercase tracking-widest transition-all shadow-sm hover:text-amber-500 whitespace-nowrap"
        title={$_("page__sprint")}
      >
        <Flag size={16} />
        <span class="hidden lg:inline">{$_("page__sprint")}</span>
      </button>
    {/if}

    {#if showSummary}
      <button
        on:click={openMonthlySummary}
        class="flex items-center gap-2 px-4 h-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-400 font-bold text-[11px] uppercase tracking-widest transition-all shadow-sm hover:text-purple-500 whitespace-nowrap"
        title={$_("page__summary_30_days")}
      >
        <CalendarDays size={16} />
        <span class="hidden lg:inline">{$_("page__summary_30_days")}</span>
      </button>
    {/if}

    {#if showDailyReflect}
      <button
        on:click={openDailyReflect}
        class="flex items-center gap-2 px-4 h-12 bg-blue-500/5 border border-blue-500/10 rounded-xl hover:bg-blue-500/10 text-blue-600 dark:text-blue-400 font-bold text-[11px] uppercase tracking-widest transition-all shadow-sm whitespace-nowrap"
        title={$_("dailyReflect__btn_open")}
      >
        <MessageSquareQuote size={16} />
        <span class="hidden lg:inline">{$_("dailyReflect__btn_open")}</span>
      </button>
    {/if}

    {#if isOwner && showMilestones}
      <button
        on:click={openMilestoneManager}
        class="flex items-center gap-2 px-4 h-12 bg-indigo-500/5 border border-indigo-500/10 rounded-xl hover:bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-bold text-[11px] uppercase tracking-widest transition-all shadow-sm whitespace-nowrap"
        title={$_("milestone.add_btn")}
      >
        <Rocket size={16} />
        <span class="hidden lg:inline">{$_("milestone.btn_countdown")}</span>
      </button>
    {/if}
  </div>
</div>
