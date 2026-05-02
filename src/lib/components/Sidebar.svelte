<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { onMount } from "svelte";
  import { user } from "$lib/stores/auth";
  import {
    currentWorkspaceName,
    currentWorkspaceColor,
    currentWorkspaceIcon,
    clearWorkspaceId,
    setWorkspaceId,
    MY_TASKS_WORKSPACE_ID,
  } from "$lib/stores/workspace";
  import { theme } from "$lib/stores/theme";
  import { locale, changeLocale } from "$lib/i18n";
  import { _ } from "svelte-i18n";
  import { api } from "$lib/apis";
  import {
    ListTodo,
    Settings,
    Sun,
    Moon,
    Globe,
    LogOut,
    ChevronDown,
    PanelLeftClose,
    PanelLeftOpen,
    Target,
    Plus,
    Check,
    Undo2,
    LayoutDashboard,
    Search,
    SquarePen,
    X,
    ClipboardCheck,
    UserCircle2
  } from "lucide-svelte";
  import SidebarUtilityTools from "$lib/components/SidebarUtilityTools.svelte";
  import { createUIActions } from "$lib/stores/uiActions";

  const uiActions = createUIActions();

  interface Workspace {
    id: string;
    name: string;
    short_name?: string;
    room_code: string;
    owner_id: string;
    color?: string;
    icon?: string;
  }

  let showWorkspaceSwitcher = false;
  let showLanguageDropdown = false;
  let showAccountDropdown = false;
  let workspaceList: Workspace[] = [];
  let isSidebarCollapsed = false;
  let previousWorkspace: { id: string, name: string, color: string } | null = null;

  $: workspaceId = $page.params.workspace_id;
  $: routeTaskId = $page.params.task_id;
  $: isMyTasksWorkspace = activeWorkspaceId === MY_TASKS_WORKSPACE_ID || $page.url.pathname.includes('/dashboard');
  $: workspaceName =
    $currentWorkspaceName || (isMyTasksWorkspace ? $_("sidebar__focus_hub") : $_("sidebar__workspace"));
  $: workspaceColor = $currentWorkspaceColor || "#6366f1";
  $: workspaceIcon = $currentWorkspaceIcon;

  $: currentPath = $page.url.pathname;
  $: normalizedCurrentPath = normalizePath(currentPath);
  $: pathWorkspaceId = getWorkspaceIdFromPath(normalizedCurrentPath);
  $: activeWorkspaceId = pathWorkspaceId || workspaceId;
  $: sidebarWorkspaceId = activeWorkspaceId || workspaceId;
  $: activeWorkspaceSection = getWorkspaceSection(normalizedCurrentPath);
  $: isPinnedTaskPage =
    !!routeTaskId &&
    ($page.url.searchParams.get("pinned") === "true" ||
      $page.url.searchParams.get("pined") === "true" ||
      $page.url.searchParams.get("pined") === "trure");

  let pinnedTasks: any[] = [];
  let isPinnedSectionOpen = true;
  $: pinnedTaskItems = pinnedTasks.filter(t => t && typeof t === 'object' && t.id);

  function loadPinnedTasks() {
    try {
      pinnedTasks = JSON.parse(localStorage.getItem("pinned-tasks") || "[]");
    } catch {
      pinnedTasks = [];
    }
  }

  function isActive(path: string): boolean {
    return normalizedCurrentPath === normalizePath(path);
  }

  function normalizePath(path: string): string {
    let normalized = path.replace(/\/+$/, "") || "/";
    if (base && normalized.startsWith(base)) {
      normalized = normalized.slice(base.length) || "/";
    }
    return normalized;
  }

  function getWorkspaceIdFromPath(path: string): string {
    return path.match(/^\/workspace\/([^/]+)(?:\/|$)/)?.[1] || "";
  }

  function getWorkspaceSection(path: string): string {
    const match = path.match(/^\/workspace\/[^/]+(?:\/([^/]+))?/);
    return match?.[1] || "";
  }

  function isDashboardActive(): boolean {
    return normalizedCurrentPath === "/dashboard";
  }

  function isMyTasksActive(): boolean {
    return activeWorkspaceId === MY_TASKS_WORKSPACE_ID && !activeWorkspaceSection;
  }

  function isWorkspaceOverviewActive(): boolean {
    return (
      activeWorkspaceId !== MY_TASKS_WORKSPACE_ID &&
      activeWorkspaceSection === "overview"
    );
  }

  function isWorkspaceBoardActive(): boolean {
    return (
      activeWorkspaceId !== MY_TASKS_WORKSPACE_ID &&
      !activeWorkspaceSection
    );
  }

  function isWorkspaceTestCasesActive(): boolean {
    return (
      activeWorkspaceId !== MY_TASKS_WORKSPACE_ID &&
      activeWorkspaceSection === "test-cases"
    );
  }

  function isPinnedTaskActive(task: any): boolean {
    return normalizedCurrentPath === getPinnedTaskPath(task);
  }

  function getPinnedTaskPath(task: any): string {
    if (!task?.workspace_id || !task?.id) return "";
    return `/workspace/${task.workspace_id}/task/${task.id}`;
  }

  function getPinnedTaskNumber(task: any): string {
    if (!task?.task_number) return "";
    const prefix = task.workspace_short_name || task.short_name || "TASK";
    return `${prefix}-${task.task_number}`;
  }

  function getPinnedTaskTitle(task: any): string {
    const taskNumber = getPinnedTaskNumber(task);
    return taskNumber ? `${taskNumber} ${task.title || ""}` : task.title || "";
  }

  function removePinnedTask(task: any) {
    pinnedTasks = pinnedTasks.filter(
      (item) =>
        !(
          item &&
          typeof item === "object" &&
          String(item.id) === String(task.id) &&
          String(item.workspace_id) === String(task.workspace_id)
        ),
    );
    localStorage.setItem("pinned-tasks", JSON.stringify(pinnedTasks));
    window.dispatchEvent(new CustomEvent("pinnedTasksChanged"));
  }

  async function loadWorkspaces() {
    try {
      const res = await api.workspaces.getList();
      if (res.ok) {
        const data = await res.json();
        workspaceList = data.workspaces || [];
      }
    } catch {}
  }

  function workspaceUrl(ws: Workspace | { id: string, room_code?: string }) {
    if (ws.room_code) {
      return `${base}/workspace/${ws.id}?room=${ws.room_code}`;
    }
    return `${base}/workspace/${ws.id}`;
  }

  function prepareWorkspaceSwitch(ws: Workspace | { id: string, name: string, color: string, short_name?: string, owner_id?: string, icon?: string, room_code?: string }) {
    if (workspaceId && workspaceId !== ws.id) {
      previousWorkspace = {
        id: workspaceId,
        name: workspaceName,
        color: workspaceColor
      };
    }
    
    closeAllDropdowns();
    setWorkspaceId(ws.id, ws.name, ws.owner_id || "", ws.color, ws.icon || "", ws.short_name || "");
    if (ws.room_code) {
      localStorage.setItem("sync-room-code", ws.room_code);
    }
    localStorage.setItem(
      "backend-server-url",
      import.meta.env.VITE_SERVER_URL || "http://127.0.0.1:3002",
    );
  }

  function handleBackToPrevious() {
    if (!previousWorkspace) return;
    const target = workspaceList.find(w => w.id === previousWorkspace?.id);
    if (target) {
      prepareWorkspaceSwitch(target);
      goto(workspaceUrl(target));
    } else if (previousWorkspace.id === MY_TASKS_WORKSPACE_ID) {
      prepareWorkspaceSwitch({
        id: MY_TASKS_WORKSPACE_ID,
        name: "Focus Hub",
        color: "#6366f1",
        icon: "Target"
      });
      goto(`${base}/workspace/${MY_TASKS_WORKSPACE_ID}${$page.url.search}`);
    }
  }

  async function handleLogout() {
    try {
      await api.auth.logout();
      document.cookie = "_khun_ph_token=; path=/; max-age=0; samesite=Lax";
      localStorage.removeItem("user_email");
      localStorage.removeItem("sync-room-code");
      localStorage.removeItem("backend-server-url");
      clearWorkspaceId();
      user.set(null);
      goto(`${base}/login`);
    } catch (e) {
      console.error("Logout failed:", e);
    }
  }

  function getInitials(name: string): string {
    return name.trim().charAt(0).toUpperCase();
  }

  function closeAllDropdowns() {
    showWorkspaceSwitcher = false;
    showLanguageDropdown = false;
    showAccountDropdown = false;
  }

  function toggleSidebar() {
    isSidebarCollapsed = !isSidebarCollapsed;
    localStorage.setItem("sidebar-collapsed", String(isSidebarCollapsed));
  }

  onMount(() => {
    loadWorkspaces();
    isSidebarCollapsed = localStorage.getItem("sidebar-collapsed") === "true";
    loadPinnedTasks();

    const handlePinnedTasksChange = () => {
      loadPinnedTasks();
    };
    window.addEventListener('pinnedTasksChanged', handlePinnedTasksChange);
    
    return () => {
      window.removeEventListener('pinnedTasksChanged', handlePinnedTasksChange);
    };
  });

  $: workspaceHref = isMyTasksWorkspace
    ? `${base}/workspace/${MY_TASKS_WORKSPACE_ID}${$page.url.search}`
    : `${base}/workspace/${sidebarWorkspaceId}${$page.url.search}`;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if showWorkspaceSwitcher || showLanguageDropdown || showAccountDropdown}
  <div class="fixed inset-0 z-30" on:click={closeAllDropdowns}></div>
{/if}

<aside
  class="flex flex-col shrink-0 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 z-40 transition-all duration-300 ease-in-out {isSidebarCollapsed ? 'w-20' : 'w-64'} h-screen"
>
  <!-- Sidebar Header -->
  <div class="p-3 border-b border-gray-200 dark:border-gray-800 relative">
    {#if !isSidebarCollapsed}
      <div class="flex items-center justify-between mb-2">
        <div class="text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em] px-2">
          {$_("sidebar__workspace")}
        </div>
        <div class="flex items-center gap-1">
          {#if previousWorkspace}
            <button
              on:click={handleBackToPrevious}
              class="p-1.5 rounded-lg text-slate-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-500/10 transition-colors"
              title={$_("sidebar__back_to", { values: { name: previousWorkspace.name } })}
            >
              <Undo2 size={18} />
            </button>
          {/if}
          <button
            on:click={toggleSidebar}
            class="p-1.5 rounded-lg text-slate-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-500/10 transition-colors"
            title={isSidebarCollapsed ? $_("sidebar__expand") : $_("sidebar__collapse")}
          >
            <PanelLeftClose size={18} class="transition-transform duration-300 {isSidebarCollapsed ? 'rotate-180' : ''}" />
          </button>
        </div>
      </div>

      <div class="relative">
        <button
          type="button"
          on:click|stopPropagation={() => {
            showWorkspaceSwitcher = !showWorkspaceSwitcher;
            showLanguageDropdown = false;
            showAccountDropdown = false;
          }}
          class="w-full flex items-center gap-2.5 px-3 py-2.5 rounded-xl bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 hover:border-indigo-300 dark:hover:border-indigo-700 transition-all text-left group"
        >
          <div
            class="w-7 h-7 rounded-lg flex items-center justify-center text-white text-[11px] font-black shrink-0 shadow-sm"
            style="background-color: {workspaceColor}"
          >
            {#if workspaceIcon === "Target"}
              <Target size={14} />
            {:else}
              {getInitials(workspaceName)}
            {/if}
          </div>
          <span class="flex-1 truncate text-xs font-black text-gray-900 dark:text-gray-100 uppercase tracking-wide">
            {workspaceName}
          </span>
          <ChevronDown size={14} class="text-gray-500 group-hover:text-indigo-500 transition-colors shrink-0" />
        </button>

        {#if showWorkspaceSwitcher}
          <div
            class="absolute left-0 top-full mt-2 w-full min-w-[240px] bg-white/95 dark:bg-gray-800/95 backdrop-blur-xl rounded-2xl shadow-2xl border border-gray-200/50 dark:border-gray-700/50 py-2 z-50 overflow-hidden animate-fade-in"
          >
            <p class="px-4 pt-2 pb-1 text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em]">
              {$_("sidebar__workspaces_switcher")}
            </p>
            <div class="px-2 pb-1">
              <a
                href="{base}/workspace/{MY_TASKS_WORKSPACE_ID}"
                on:click={() =>
                  prepareWorkspaceSwitch({
                    id: MY_TASKS_WORKSPACE_ID,
                    name: $_("sidebar__focus_hub"),
                    color: "#6366f1",
                    icon: "Target",
                  })}
                class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-gray-700 dark:text-gray-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 hover:text-indigo-600 dark:hover:text-indigo-400 transition-all text-left"
              >
                <div
                  class="w-5 h-5 rounded flex items-center justify-center text-white text-[10px] font-bold shrink-0"
                  style="background-color: #6366f1"
                >
                  <Target size={12} />
                </div>
                <span class="flex-1 truncate font-medium">{$_("sidebar__focus_hub")}</span>
                {#if workspaceId === MY_TASKS_WORKSPACE_ID || isMyTasksWorkspace}
                  <Check size={14} class="text-indigo-500 shrink-0" />
                {/if}
              </a>
              {#each workspaceList as ws}
                <a
                  href={workspaceUrl(ws)}
                  on:click={() => prepareWorkspaceSwitch(ws)}
                  class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-gray-700 dark:text-gray-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 hover:text-indigo-600 dark:hover:text-indigo-400 transition-all text-left"
                >
                  <div
                    class="w-5 h-5 rounded flex items-center justify-center text-white text-[10px] font-bold shrink-0"
                    style="background-color: {ws.color || '#6366f1'}"
                  >
                    {getInitials(ws.name)}
                  </div>
                  <span class="flex-1 truncate font-medium">{ws.name}</span>
                  {#if ws.id === workspaceId}
                    <Check size={14} class="text-indigo-500 shrink-0" />
                  {/if}
                </a>
              {/each}
            </div>

            <div class="border-t border-gray-100 dark:border-gray-700 px-2 mt-1 pt-1">
              <a
                href="{base}/dashboard"
                on:click={closeAllDropdowns}
                class="flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
              >
                <Plus size={14} />
                <span class="font-bold">{$_("sidebar__create_workspace")}</span>
              </a>
            </div>
          </div>
        {/if}
      </div>

      <!-- Quick Actions -->
      <div class="mt-4 space-y-1">
        <button
          on:click={async () => {
            if ($page.url.pathname.includes('/task/')) {
              await goto(`${base}/workspace/${workspaceId}`);
            }
            uiActions.openModal("commandPalette");
          }}
          class="w-full flex items-center justify-between px-3 py-2 rounded-xl text-sm text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors text-left group"
        >
          <div class="flex items-center gap-2.5">
            <Search size={16} class="shrink-0" />
            <span class="font-medium">{$_("sidebar__search")}</span>
          </div>
          <kbd class="px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700/50 bg-gray-50 dark:bg-gray-800/50 text-[10px] font-medium text-gray-400 dark:text-gray-500">⌘K</kbd>
        </button>
        {#if !isMyTasksWorkspace}
          <button
            on:click={async () => {
              if ($page.url.pathname.includes('/task/')) {
                await goto(`${base}/workspace/${workspaceId}`);
              }
              uiActions.openModal("form");
            }}
            class="w-full flex items-center justify-between px-3 py-2 rounded-xl text-sm text-gray-700 dark:text-gray-300 bg-gray-100/50 dark:bg-gray-800/30 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800/50 transition-colors text-left group"
          >
            <div class="flex items-center gap-2.5">
              <SquarePen size={16} class="shrink-0" />
              <span class="font-medium">{$_("sidebar__new_issue")}</span>
            </div>
            <kbd class="px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700/50 bg-white dark:bg-gray-800/50 text-[10px] font-medium text-gray-500 dark:text-gray-400">N</kbd>
          </button>
        {/if}
      </div>
    {:else}
      <div class="flex flex-col gap-4 items-center">
        <button
          on:click={toggleSidebar}
          class="p-2 rounded-xl text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 transition-all duration-300"
          title={$_("sidebar__expand")}
        >
          <PanelLeftOpen size={20} />
        </button>
        {#if previousWorkspace}
          <button
            on:click={handleBackToPrevious}
            class="p-2 rounded-xl text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 transition-all duration-300"
            title={$_("sidebar__back_to", { values: { name: previousWorkspace.name } })}
          >
            <Undo2 size={20} />
          </button>
        {/if}
        <button
          on:click={() => {
            showWorkspaceSwitcher = !showWorkspaceSwitcher;
            showLanguageDropdown = false;
            showAccountDropdown = false;
          }}
          class="w-12 h-12 rounded-2xl flex items-center justify-center text-white text-xs font-black shrink-0 shadow-lg transition-all hover:scale-105 active:scale-90"
          style="background-color: {workspaceColor}"
        >
          {getInitials(workspaceName)}
        </button>
      </div>
    {/if}
  </div>

  <!-- Navigation -->
  <nav class="flex-1 overflow-y-auto p-3 space-y-6">
    <div>
      {#if !isSidebarCollapsed}
        <div class="px-2 py-1 text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em] mb-2">
          {$_("sidebar__personal")}
        </div>
      {/if}
      <div class="space-y-1">
        <a
          href="{base}/dashboard"
          title={$_("sidebar__dashboard")}
          class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isDashboardActive()
            ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
            : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
        >
          {#if isDashboardActive()}
            <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
          {/if}
          <LayoutDashboard size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
          {#if !isSidebarCollapsed}
            <span>{$_("sidebar__dashboard")}</span>
          {/if}
        </a>
        <a
          href="{base}/workspace/{MY_TASKS_WORKSPACE_ID}"
          title={$_("sidebar__my_tasks")}
          class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isMyTasksActive()
            ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
            : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
        >
          {#if isMyTasksActive()}
            <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
          {/if}
          <Target size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
          {#if !isSidebarCollapsed}
            <span>{$_("sidebar__my_tasks")}</span>
          {/if}
        </a>
      </div>
    </div>

    <!-- Pinned Tasks -->
    {#if pinnedTaskItems.length > 0}
      <div>
        {#if !isSidebarCollapsed}
          <button
            on:click={() => isPinnedSectionOpen = !isPinnedSectionOpen}
            class="w-full flex items-center gap-1 px-2 py-1 text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em] mb-2 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          >
            <span>Pinned</span>
            <ChevronDown size={12} class="transition-transform duration-200 {isPinnedSectionOpen ? '' : '-rotate-90'}" />
            <span class="ml-auto text-[10px] font-bold tracking-normal text-gray-400 dark:text-gray-500">{pinnedTaskItems.length}</span>
          </button>
        {/if}
        {#if isPinnedSectionOpen || isSidebarCollapsed}
          <div class="space-y-1">
            {#each pinnedTaskItems as task (`${task.workspace_id}:${task.id}`)}
              <div
                class="group flex items-center gap-2 rounded-xl text-sm transition-all duration-300 relative {isPinnedTaskActive(task)
                  ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
                  : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
              >
                {#if isPinnedTaskActive(task)}
                  <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
                {/if}
                <a
                  href="{base}/workspace/{task.workspace_id}/task/{task.id}?pinned=true"
                  title={getPinnedTaskTitle(task)}
                  class="min-w-0 flex-1 flex items-center gap-2.5 py-2 {isSidebarCollapsed ? 'justify-center px-0' : 'pl-3'}"
                >
                  <div class="w-2 h-2 rounded-full border border-gray-400 shrink-0"></div>
                  {#if !isSidebarCollapsed}
                    <span class="truncate">
                      {#if getPinnedTaskNumber(task)}
                        <span class="text-gray-400 font-normal mr-1">{getPinnedTaskNumber(task)}</span>
                      {/if}
                      {task.title}
                    </span>
                  {/if}
                </a>
                {#if !isSidebarCollapsed}
                  <button
                    type="button"
                    on:click|stopPropagation={() => removePinnedTask(task)}
                    class="mr-2 p-1 rounded-md text-gray-400 opacity-0 transition-all hover:bg-gray-200/60 hover:text-gray-700 group-hover:opacity-100 dark:hover:bg-gray-700/70 dark:hover:text-gray-200"
                    title="Unpin"
                    aria-label="Unpin {getPinnedTaskTitle(task)}"
                  >
                    <X size={14} />
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Workspace -->
    {#if !isPinnedTaskPage && !isMyTasksWorkspace && sidebarWorkspaceId && sidebarWorkspaceId !== MY_TASKS_WORKSPACE_ID}
      <div>
        {#if !isSidebarCollapsed}
          <div class="px-2 py-1 text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em] mb-2">
            {$_("sidebar__workspace")}
          </div>
        {/if}
        <div class="space-y-1">
          <a
            href="{base}/workspace/{sidebarWorkspaceId}/overview{$page.url.search}"
            title={$_("sidebar__overview")}
            class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isWorkspaceOverviewActive()
              ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
              : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
          >
            {#if isWorkspaceOverviewActive()}
              <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
            {/if}
            <LayoutDashboard size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
            {#if !isSidebarCollapsed}
              <span>{$_("sidebar__overview")}</span>
            {/if}
          </a>
          <a
            href={workspaceHref}
            title={$_("sidebar__issues")}
            class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isWorkspaceBoardActive()
              ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
              : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
          >
            {#if isWorkspaceBoardActive()}
              <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
            {/if}
            <ListTodo size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
            {#if !isSidebarCollapsed}
              <span>{$_("sidebar__issues")}</span>
            {/if}
          </a>
          <a
            href="{base}/workspace/{sidebarWorkspaceId}/test-cases{$page.url.search}"
            title={$_("sidebar__test_cases")}
            class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isWorkspaceTestCasesActive()
              ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
              : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
          >
            {#if isWorkspaceTestCasesActive()}
              <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
            {/if}
            <ClipboardCheck size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
            {#if !isSidebarCollapsed}
              <span>{$_("sidebar__test_cases")}</span>
            {/if}
          </a>
        </div>
      </div>
    {/if}

    <div>
      <SidebarUtilityTools isCollapsed={isSidebarCollapsed} />
    </div>

    <!-- Configure -->
    {#if $user?.role === "admin"}
      <div>
        {#if !isSidebarCollapsed}
          <div class="px-2 py-1 text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em] mb-2">
            {$_("sidebar__configure")}
          </div>
        {/if}
        <div class="space-y-1">
          <a
            href="{base}/settings"
            title={$_("sidebar__settings")}
            class="group flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative {isActive(`${base}/settings`)
              ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 font-black shadow-[inset_0_0_12px_rgba(99,102,241,0.05)]'
              : 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-900 dark:hover:text-white'} {isSidebarCollapsed ? 'justify-center px-0' : ''}"
          >
            {#if isActive(`${base}/settings`)}
              <div class="absolute left-0 w-1 h-4 bg-indigo-500 rounded-r-full"></div>
            {/if}
            <Settings size={isSidebarCollapsed ? 22 : 18} class="shrink-0 transition-transform group-hover:scale-110" />
            {#if !isSidebarCollapsed}
              <span>{$_("sidebar__settings")}</span>
            {/if}
          </a>
        </div>
      </div>
    {/if}
  </nav>

  <!-- Footer -->
  <div
    class="p-4 border-t border-gray-200 dark:border-gray-800 flex items-center relative {isSidebarCollapsed ? 'flex-col gap-6' : 'gap-1'}"
  >
    {#if !isSidebarCollapsed}
      <!-- Theme Toggle -->
      <button
        on:click={() => theme.toggle()}
        class="p-2 rounded-xl text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title={$_("layout__change_theme")}
      >
        {#if $theme === "light"}
          <Sun size={18} />
        {:else}
          <Moon size={18} />
        {/if}
      </button>

      <!-- Language Switcher -->
      <div class="relative">
        <button
          type="button"
          on:click|stopPropagation={() => {
            showLanguageDropdown = !showLanguageDropdown;
            showWorkspaceSwitcher = false;
            showAccountDropdown = false;
          }}
          class="p-2 rounded-xl text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1"
          title={$_("layout__change_language")}
        >
          <Globe size={18} />
          <span class="text-[10px] font-black uppercase tracking-wider">{$locale}</span>
        </button>

        {#if showLanguageDropdown}
          <div
            class="absolute bottom-full left-0 mb-2 w-40 bg-white/95 dark:bg-gray-800/95 backdrop-blur-xl rounded-2xl shadow-2xl border border-gray-200/50 dark:border-gray-700/50 py-1.5 z-50 overflow-hidden animate-fade-in"
          >
            <button
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 flex items-center gap-2 {$locale ===
              'th'
                ? 'text-indigo-600 font-bold'
                : ''}"
              on:click={() => {
                changeLocale("th");
                showLanguageDropdown = false;
              }}
            >
              <span>🇹🇭</span> ไทย
            </button>
            <button
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 flex items-center gap-2 {$locale ===
              'en'
                ? 'text-indigo-600 font-bold'
                : ''}"
              on:click={() => {
                changeLocale("en");
                showLanguageDropdown = false;
              }}
            >
              <span>🇺🇸</span> English
            </button>
          </div>
        {/if}
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>
    {/if}

    <!-- User Avatar -->
    {#if $user}
      <div>
        <button
          type="button"
          on:click|stopPropagation={() => {
            showAccountDropdown = !showAccountDropdown;
            showWorkspaceSwitcher = false;
            showLanguageDropdown = false;
          }}
          class="w-9 h-9 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-black text-xs hover:ring-2 hover:ring-indigo-400 transition-all shadow-lg active:scale-90"
          title={$user.profile?.nickname || $user.email}
        >
          {getInitials($user.profile?.nickname || $user.email.split("@")[0])}
        </button>

        {#if showAccountDropdown}
          <div
            class="absolute bottom-full mb-2 bg-white/95 dark:bg-gray-800/95 backdrop-blur-xl rounded-2xl shadow-2xl border border-gray-200/50 dark:border-gray-700/50 py-1.5 z-50 overflow-hidden animate-fade-in {isSidebarCollapsed ? 'left-full ml-2 w-52' : 'left-0 w-full'}"
          >
            <div class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50">
              <p class="text-sm font-black text-gray-900 dark:text-white truncate">
                {$user.profile?.nickname || $user.email.split("@")[0]}
              </p>
              <p class="text-[10px] font-bold text-gray-500 dark:text-gray-400 truncate tracking-wide">
                {$user.email}
              </p>
            </div>
            <div class="p-1">
              <a
                href="{base}/profile"
                on:click={closeAllDropdowns}
                class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5 transition-colors font-semibold"
              >
                <UserCircle2 size={16} />
                <span>{$_("layout__profile_settings")}</span>
              </a>
              <button
                on:click={() => {
                  closeAllDropdowns();
                  handleLogout();
                }}
                class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors font-bold"
              >
                <LogOut size={16} />
                <span>{$_("layout__logout")}</span>
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</aside>
