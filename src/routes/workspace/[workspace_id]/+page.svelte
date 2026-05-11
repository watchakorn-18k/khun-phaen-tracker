<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { get } from "svelte/store";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { api } from "$lib/apis";
  import { _ } from "svelte-i18n";
  import type { Task, FilterOptions, AssigneeGroup } from "$lib/types";
  import type { Milestone } from "$lib/types/milestone";
  import { getAssigneeGroups } from "$lib/db";
  import { initWasmSearch, indexTasks, wasmReady } from "$lib/stores/search";
  import { connectRealtime, disconnectRealtime } from "$lib/stores/realtime";
  import { user } from "$lib/stores/auth";
  import {
    MY_TASKS_WORKSPACE_ID,
    currentWorkspaceOwnerId,
    currentWorkspaceName,
    setWorkspaceId,
  } from "$lib/stores/workspace";
  import { theme } from "$lib/stores/theme";
  import { sprints, type Sprint } from "$lib/stores/sprintStore";
  import {
    DEFAULT_FILTERS,
    persistFilters,
    restoreFilters,
    clearSavedFilters,
    normalizeSprintFilterValue,
  } from "$lib/stores/filterActions";
  import { createKeyboardHandler } from "$lib/stores/keyboardActions";
  import { modals, toast } from "$lib/stores/uiActions";
  import { createWorkspacePageStore } from "$lib/stores/workspacePageStore";
  import { getKeyboardConfig } from "$lib/stores/workspaceKeyboardConfig";
  import { buildMonthlySummary } from "$lib/utils/monthly-summary";
  import SearchAndActions from "$lib/components/SearchAndActions.svelte";
  import ViewSelector from "$lib/components/ViewSelector.svelte";
  import FilterPanel from "$lib/components/FilterPanel.svelte";
  import WorkspaceViews from "$lib/components/WorkspaceViews.svelte";
  import WorkspaceModals from "$lib/components/WorkspaceModals.svelte";
  import StatsPanel from "$lib/components/StatsPanel.svelte";
  import StatusToast from "$lib/components/StatusToast.svelte";
  import ExportProgress from "$lib/components/ExportProgress.svelte";
  import AccessDenied from "$lib/components/AccessDenied.svelte";
  import WorkspaceLoading from "$lib/components/WorkspaceLoading.svelte";
  import DailyReflect from "$lib/components/DailyReflect.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import MilestoneCountdown from "$lib/components/MilestoneCountdown.svelte";

  const ws = createWorkspacePageStore();
  const {
    loadingData,
    tasks,
    allTasksIncludingArchived,
    filteredTasks,
    categories,
    projects,
    projectList,
    assignees,
    workerStats,
    projectStats,
    stats,
    totalTasks,
    totalPages,
    hasAccess,
    checkingAccess,
    filters,
    searchInput,
    viewActions,
    uiActions,
    searchActions,
    workspaceActions,
    taskActions,
    sprintActions,
    exportActions,
    loadData,
    debouncedLoadData,
    videoExportState,
    editingTask,
  } = ws;

  const { currentView, pageSize, currentPage } = viewActions;

  let monthlySummaryRef: HTMLDivElement | null = null;
  let searchInputRef: HTMLInputElement | null = null;
  let isKanbanDragging = false;
  let qrExportTasks: Task[] = [];
  let newPageSize = 20;
  let milestones: Milestone[] = [];
  let editingMilestone: Milestone | null = null;
  let assigneeGroups: AssigneeGroup[] = [];
  let suppressTaskAutoOpen = false;
  let activeWorkspaceKey = "";
  let workspaceInitRun = 0;
  let aiChatOpen = false;
  let aiChatInput = "";
  let aiChatLoading = false;
  let aiChatError = "";
  let aiChatMessages: Array<{
    role: "user" | "assistant";
    content: string;
    results?: Array<{ task: Task; score: number }>;
  }> = [];
  let filterDropdownEl: HTMLDivElement | null = null;
  let filterPanelEl: HTMLDivElement | null = null;
  let filterDropdownPos = { top: 0, left: 0 };
  type SavedMyTasksFilter = {
    enabled: boolean;
    assignee_id: string | number | null;
  };
  const MY_TASKS_FILTER_STORAGE_KEY = "workspace-my-tasks-filter";
  $: isMyTasksWorkspace = $page.params.workspace_id === MY_TASKS_WORKSPACE_ID;
  $: visibleMilestones = milestones.filter((m) => !m.is_hidden);

  async function loadAssigneeGroups() {
    try {
      assigneeGroups = await getAssigneeGroups(true);
    } catch (e) {
      console.error("Failed to load assignee groups:", e);
      assigneeGroups = [];
    }
  }

  async function fetchMilestones() {
    const wsId = $page.params.workspace_id;
    if (!wsId || wsId === MY_TASKS_WORKSPACE_ID) return;
    try {
      const res = await api.workspaces.getMilestones(wsId);
      if (res.ok) {
        const data = await res.json();
        // Normalize IDs and workspace IDs from MongoDB (if sent as $oid object)
        milestones = data.map((m: any) => ({
          ...m,
          id: m.id || m._id || "",
          workspace_id:
            typeof m.workspace_id === "object" && m.workspace_id.$oid
              ? m.workspace_id.$oid
              : m.workspace_id,
        }));
      }
    } catch (e) {}
  }

  function openTaskPage(task: Task) {
    const wsId = $page.params.workspace_id;
    const urlRoom = $page.url.searchParams.get("room");
    const roomParam = urlRoom ? `?room=${urlRoom}` : "";
    goto(`${base}/workspace/${wsId}/task/${task.id}${roomParam}`);
  }

  function handleWindowClick(event: MouseEvent) {
    if (!$modals.filters) return;
    const target = event.target as Node;
    if (filterDropdownEl?.contains(target)) return;
    if (filterPanelEl?.contains(target)) return;
    uiActions.closeModal("filters");
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) node.parentNode.removeChild(node);
      },
    };
  }

  function handleToggleFilters(event: CustomEvent<{ top: number; left: number } | null>) {
    if (event.detail) filterDropdownPos = event.detail;
    uiActions.toggleModal("filters");
  }

  // Auto-redirect task from old URL format to new page
  $: if (browser && !$loadingData && $allTasksIncludingArchived.length > 0) {
    const urlTaskId = $page.url.searchParams.get("task");
    if (urlTaskId) {
      const task = $allTasksIncludingArchived.find(
        (t) => String(t.id) === urlTaskId,
      );
      if (task) {
        openTaskPage(task);
      }
    }
  }

  $: myAssigneeId = $assignees.find(
    (a) => a.user_id === $user?.id || a.user_id === $user?.user_id,
  )?.id;
  $: myTasksStorageKey = `${MY_TASKS_FILTER_STORAGE_KEY}:${$page.params.workspace_id}:${$user?.id || $user?.user_id || "anon"}`;
  $: isMyTasksActive =
    myAssigneeId !== undefined &&
    myAssigneeId !== null &&
    String($filters?.assignee_id) === String(myAssigneeId);
  $: isOwner =
    !isMyTasksWorkspace && $currentWorkspaceOwnerId && $user?.id
      ? $currentWorkspaceOwnerId === $user.id
      : false;
  $: isAdminOrOwner = isOwner || $user?.role === "admin";

  import { tabSettings } from "$lib/stores/tabSettings";
  $: restrictedTabIds = [
    ...(isMyTasksWorkspace ? ["workload"] : []),
    ...(!isAdminOrOwner && !isMyTasksWorkspace ? ["gantt", "workload"] : []),
  ] as import("$lib/stores/tabSettings").TabId[];
  $: visibleTabs = $tabSettings
    .filter((t) => t.enabled !== false)
    .filter((t) => !restrictedTabIds.includes(t.id))
    .map((t) => ({ id: t.id, icon: t.icon }));

  $: if (browser && $hasAccess && !$checkingAccess && $filters) {
    const normalized = normalizeSprintFilterValue($filters.sprint_id, $sprints);
    if ($filters.sprint_id !== normalized)
      filters.update((f) => ({ ...f, sprint_id: normalized }));
    else {
      persistFilters($filters);
      currentPage.set(1);
      debouncedLoadData();
    }
  }

  const keyboardHandler = createKeyboardHandler(
    getKeyboardConfig({
      uiActions,
      modals,
      editingTask: $editingTask,
      setEditingTask: (v) => ($editingTask = v),
      searchInputRef,
      visibleTabs,
      viewActions,
    }),
  );

  async function initializeWorkspace(workspaceId: string, urlRoom: string | null) {
    const runId = ++workspaceInitRun;
    if (!workspaceId) return goto(`${base}/dashboard`);

    checkingAccess.set(true);
    hasAccess.set(false);
    milestones = [];
    editingMilestone = null;
    assigneeGroups = [];
    $editingTask = null;
    disconnectRealtime();

    if (workspaceId === MY_TASKS_WORKSPACE_ID) {
      if (runId !== workspaceInitRun) return;
      checkingAccess.set(false);
      hasAccess.set(true);
      setWorkspaceId(
        workspaceId,
        "โฟกัสฮับ",
        "",
        "#0f766e",
        "Target",
        "FOCUS",
      );
      if (get(currentView) === "workload") {
        currentView.set("table");
      }
      await loadAssigneeGroups();
      if (runId !== workspaceInitRun) return;
      initWasmSearch();
      filters.set({ ...DEFAULT_FILTERS, sprint_id: "all", assignee_id: "all" });
      await loadData();
      return;
    }
    if (!urlRoom) return goto(`${base}/dashboard`);
    try {
      const res = await api.workspaces.checkAccess(urlRoom);
      const data = await res.json();
      if (runId !== workspaceInitRun) return;
      checkingAccess.set(false);
      if (data.success && data.has_access) {
        hasAccess.set(true);
        setWorkspaceId(
          workspaceId,
          data.workspace?.name || "",
          data.workspace?.owner_id || "",
          data.workspace?.color,
          data.workspace?.icon,
          data.workspace?.short_name,
        );
        connectRealtime(urlRoom, (payload) =>
          workspaceActions.handleRealtimeUpdate(payload),
        );
        await sprints.refresh();
        await fetchMilestones();
        await loadAssigneeGroups();
        const restoredFilters = restoreFilters($sprints);
        let initialFilters = { ...restoredFilters };
        const savedMyTasksRaw = localStorage.getItem(myTasksStorageKey);
        if (savedMyTasksRaw) {
          try {
            const savedMyTasks = JSON.parse(
              savedMyTasksRaw,
            ) as SavedMyTasksFilter;
            if (savedMyTasks.enabled && savedMyTasks.assignee_id !== null) {
              initialFilters = {
                ...initialFilters,
                assignee_id: savedMyTasks.assignee_id,
              };
            }
          } catch {}
        }
        filters.set(initialFilters);
        initWasmSearch();
      } else {
        hasAccess.set(false);
      }
    } catch (err) {
      if (runId !== workspaceInitRun) return;
      checkingAccess.set(false);
      hasAccess.set(false);
    }
  }

  $: if (browser) {
    const workspaceId = $page.params.workspace_id;
    const urlRoom = $page.url.searchParams.get("room");
    const nextWorkspaceKey = `${workspaceId || ""}:${urlRoom || ""}`;
    if (workspaceId && nextWorkspaceKey !== activeWorkspaceKey) {
      activeWorkspaceKey = nextWorkspaceKey;
      void initializeWorkspace(workspaceId, urlRoom);
    }
  }

  onMount(() => {
    document.addEventListener("keydown", keyboardHandler);
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("keydown", keyboardHandler);
      disconnectRealtime();
    }
  });

  $: if ($wasmReady) indexTasks($tasks);
  $: monthlySummary = buildMonthlySummary($allTasksIncludingArchived);
  $: sprintManagerTasks = $filteredTasks;

  function cancelEdit() {
    suppressTaskAutoOpen = true;
    $editingTask = null;
    uiActions.closeModal("form");
    updateUrlTask(null);
  }
  function updateUrlTask(id: any) {
    if (!browser) return;
    const url = new URL(window.location.href);
    if (id) url.searchParams.set("task", String(id));
    else url.searchParams.delete("task");
    goto(url.toString(), { replaceState: true, noScroll: true });
  }

  function toggleMyTasksFilter() {
    if (myAssigneeId === undefined || myAssigneeId === null) return;
    filters.update((f) => {
      const nextIsActive = f.assignee_id !== myAssigneeId;
      if (browser) {
        const savedPayload: SavedMyTasksFilter = nextIsActive
          ? { enabled: true, assignee_id: myAssigneeId }
          : { enabled: false, assignee_id: null };
        localStorage.setItem(myTasksStorageKey, JSON.stringify(savedPayload));
      }
      return {
        ...f,
        assignee_id: nextIsActive ? myAssigneeId : "all",
      };
    });
  }

  async function sendAiChat() {
    const message = aiChatInput.trim();
    const workspaceId = $page.params.workspace_id;
    if (!message || !workspaceId || aiChatLoading) return;

    aiChatMessages = [...aiChatMessages, { role: "user", content: message }];
    aiChatInput = "";
    aiChatLoading = true;
    aiChatError = "";

    try {
      const res = await api.data.ai.chatTasks(workspaceId, {
        message,
        limit: 5,
        task_limit: 100,
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || "AI chat failed");
      aiChatMessages = [
        ...aiChatMessages,
        {
          role: "assistant",
          content: data.answer || "เจอ task ที่เกี่ยวข้อง",
          results: data.results || [],
        },
      ];
    } catch (error) {
      aiChatError = error instanceof Error ? error.message : "AI chat failed";
    } finally {
      aiChatLoading = false;
    }
  }

  function openAiResult(task: Task) {
    openTaskPage(task);
    aiChatOpen = false;
  }
</script>

<svelte:head>
  <title>{$_("meta__workspace_title", { values: { name: $currentWorkspaceName || "Khun Phaen" } })}</title>
  <meta name="description" content={$_("meta__workspace_desc", { values: { name: $currentWorkspaceName || "Khun Phaen" } })} />
</svelte:head>

<svelte:window on:click={handleWindowClick} />

<div class="px-4 sm:px-6 space-y-6 pb-24 pt-4">
  {#if $checkingAccess}
    <WorkspaceLoading />
  {:else if !$hasAccess}
    <AccessDenied />
  {:else}
    <ExportProgress
      inProgress={$videoExportState.inProgress}
      percent={$videoExportState.percent}
      elapsedMs={$videoExportState.elapsedMs}
    />
    <StatusToast
      message={$toast?.message || ""}
      type={$toast?.type || "success"}
    />

    {#if visibleMilestones.length > 0}
      <div class="grid gap-6">
        {#each visibleMilestones as milestone}
          <MilestoneCountdown
            {milestone}
            {isOwner}
            onEdit={(m) => {
              editingMilestone = m;
              uiActions.openModal("milestoneManager");
            }}
            onDelete={() => uiActions.openModal("milestoneManager")}
            onHide={fetchMilestones}
          />
        {/each}
      </div>
    {/if}
    <StatsPanel
      stats={$stats}
      {isOwner}
      {videoExportState}
      on:filterStatus={(e) => {
        filters.update((f) => ({ ...f, status: e.detail.status }));
      }}
      on:exportCSV={() => exportActions.handleExportCSV()}
      on:exportPDF={() => exportActions.handleExportPDF()}
      on:exportMarkdown={(e) => exportActions.handleExportMarkdown(e)}
      on:exportVideo={(e) => exportActions.handleExportVideo(e)}
      on:exportSlide={(e) => exportActions.handleExportSlide(e)}
      on:exportDatabase={(e) => exportActions.handleExportDatabase(e)}
      on:importCSV={(e) => exportActions.handleImportCSV(e)}
    />
    <div class="relative z-40" bind:this={filterDropdownEl}>
      <SearchAndActions
        isFiltersOpen={$modals.filters}
        {isOwner}
        {isMyTasksActive}
        showTeam={!isMyTasksWorkspace}
        showProjects={!isMyTasksWorkspace}
        showSprints={!isMyTasksWorkspace}
        showWorkspaceSettings={!isMyTasksWorkspace}
        showMilestones={!isMyTasksWorkspace}
        on:toggleFilters={handleToggleFilters}
        on:closeFilters={() => uiActions.closeModal("filters")}
        on:openWorkerManager={() => uiActions.openModal("workerManager")}
        on:openProjectManager={() => uiActions.openModal("projectManager")}
        on:openSprintManager={() => uiActions.openModal("sprintManager")}
        on:openMonthlySummary={() => uiActions.openModal("monthlySummary")}
        on:openDailyReflect={() => uiActions.openModal("dailyReflect")}
        on:openMilestoneManager={() => uiActions.openModal("milestoneManager")}
        on:openWorkspaceSettings={() => uiActions.openModal("workspaceSettings")}
        on:toggleMyTasks={toggleMyTasksFilter}
      />

      {#if $modals.filters}
        <div
          use:portal
          bind:this={filterPanelEl}
          class="fixed z-[9999]"
          style="top: {filterDropdownPos.top}px; left: {filterDropdownPos.left}px;"
        >
          <FilterPanel
            bind:filters={$filters}
            categories={$categories}
            projects={$projects}
            assignees={$assignees}
            sprints={$sprints}
            tasks={$allTasksIncludingArchived}
            {myAssigneeId}
            dropdown
            on:clearFilters={() => {
              filters.set({ ...DEFAULT_FILTERS });
              clearSavedFilters();
            }}
          />
        </div>
      {/if}
    </div>

    <ViewSelector
      currentView={$currentView}
      {visibleTabs}
      isTabSettingsOpen={$modals.tabSettings}
      showAddTask={!isMyTasksWorkspace}
      hiddenTabIds={restrictedTabIds}
      on:switchView={(e) => viewActions.switchView(e.detail)}
      on:toggleTabSettings={() => uiActions.toggleModal("tabSettings")}
      on:closeTabSettings={() => uiActions.closeModal("tabSettings")}
      on:addTask={() => {
        uiActions.toggleModal("form");
        $editingTask = null;
        updateUrlTask(null);
      }}
    />

    <WorkspaceViews
      loadingData={$loadingData}
      filteredTasks={$filteredTasks}
      tasks={$tasks}
      currentView={$currentView}
      sprints={$sprints}
      {currentPage}
      totalPages={$totalPages}
      totalTasks={$totalTasks}
      {pageSize}
      {sprintManagerTasks}
      assignees={$assignees}
      on:statusChange={(e) => taskActions.handleStatusChange(e)}
      on:priorityChange={(e) => taskActions.handlePriorityChange(e)}
      on:edit={(e) => {
        void openTaskPage(e.detail);
      }}
      on:delete={(e) => taskActions.handleDeleteTask(e)}
      on:dragState={(e) => (isKanbanDragging = e.detail)}
      on:pageChange={(e) => viewActions.handlePageChange(e.detail, $totalPages)}
      on:pageSizeSettings={() => {
        newPageSize = $pageSize;
        uiActions.openModal("pageSize");
      }}
      on:deleteSelected={(e) => taskActions.handleDeleteSelectedTasks(e)}
      on:checklistToggle={(e) => taskActions.handleChecklistToggle(e)}
      on:exportQR={(e) => {
        qrExportTasks = $tasks.filter((t) =>
          e.detail.map(String).includes(String(t.id)),
        );
        uiActions.openModal("qrExport");
      }}
      on:selectAssignee={(e) => {
        filters.update((f) => ({ ...f, assignee_id: e.detail }));
        viewActions.switchView("list");
        loadData();
      }}
    />

    <WorkspaceModals
      modals={$modals}
      editingTask={$editingTask}
      assignees={$assignees}
      {isOwner}
      projectList={$projectList}
      sprints={$sprints}
      allTasksIncludingArchived={$allTasksIncludingArchived}
      {qrExportTasks}
      loadingData={$loadingData}
      workerStats={$workerStats}
      projectStats={$projectStats}
      {monthlySummary}
      {monthlySummaryRef}
      workspaceId={String($page.params.workspace_id || "")}
      {newPageSize}
      {editingMilestone}
      on:addTask={(e) => taskActions.handleAddTask(e)}
      on:cancelEdit={cancelEdit}
      on:addAssignee={(e) => taskActions.handleAddAssignee(e)}
      on:checklistUpdate={(e) => taskActions.handleChecklistUpdate(e)}
      on:closeModal={(e) => {
        uiActions.closeModal(e.detail);
        if (e.detail === "milestoneManager") editingMilestone = null;
      }}
      on:savePageSize={(e) => {
        newPageSize = e.detail.value;
        viewActions.setPageSize(newPageSize);
        uiActions.closeModal("pageSize");
      }}
      on:closeWorkerManager={() => uiActions.closeModal("workerManager")}
      on:addWorker={(e) => workspaceActions.handleAddWorker(e)}
      on:updateWorker={(e) => workspaceActions.handleUpdateWorker(e)}
      on:deleteWorker={(e) => workspaceActions.handleDeleteWorker(e)}
      on:groupsChanged={loadAssigneeGroups}
      on:closeSprintManager={() => uiActions.closeModal("sprintManager")}
      on:completeSprint={(e) => sprintActions.handleCompleteSprint(e)}
      on:completeAndExport={(e) => exportActions.handleCompleteAndExport(e)}
      on:deleteSprint={(e) => sprintActions.handleDeleteSprint(e)}
      on:moveTasksToSprint={(e) => sprintActions.handleMoveTasksToSprint(e)}
      on:exportMarkdown={(e) => exportActions.handleExportMarkdown(e)}
      on:exportVideo={(e) => exportActions.handleExportVideo(e)}
      on:closeProjectManager={() => uiActions.closeModal("projectManager")}
      on:addProject={(e) => workspaceActions.handleAddProject(e)}
      on:updateProject={(e) => workspaceActions.handleUpdateProject(e)}
      on:deleteProject={(e) => workspaceActions.handleDeleteProject(e)}
      on:exportMonthlyPDF={() => exportActions.handleExportMonthlyPDF()}
      on:milestonesUpdated={fetchMilestones}
    />
    <DailyReflect bind:show={$modals.dailyReflect} {isOwner} />
    {#if !isMyTasksWorkspace}
      <button
        type="button"
        class="fixed bottom-5 right-5 z-50 flex h-14 w-14 items-center justify-center rounded-2xl bg-indigo-600 text-white shadow-2xl shadow-indigo-500/30 transition hover:bg-indigo-500"
        aria-label="เปิด AI task chat"
        on:click={() => (aiChatOpen = !aiChatOpen)}
      >
        AI
      </button>

      {#if aiChatOpen}
        <div class="fixed bottom-24 right-5 z-50 w-[min(420px,calc(100vw-2.5rem))] overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-2xl dark:border-slate-700 dark:bg-slate-900">
          <div class="flex items-center justify-between border-b border-slate-200 px-4 py-3 dark:border-slate-700">
            <div>
              <div class="font-semibold text-slate-900 dark:text-white">AI Task Chat</div>
              <div class="text-xs text-slate-500 dark:text-slate-400">ค้นหา task ด้วย semantic search</div>
            </div>
            <button
              type="button"
              class="rounded-lg px-2 py-1 text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
              on:click={() => (aiChatOpen = false)}
            >
              ✕
            </button>
          </div>

          <div class="max-h-[440px] space-y-3 overflow-y-auto p-4">
            {#if aiChatMessages.length === 0}
              <div class="rounded-2xl bg-slate-100 p-3 text-sm text-slate-600 dark:bg-slate-800 dark:text-slate-300">
                ลองถามเช่น “งาน priority สูงที่ยังไม่เสร็จมีอะไรบ้าง” หรือ “task เกี่ยวกับ login”
              </div>
            {/if}

            {#each aiChatMessages as message}
              <div class:ml-8={message.role === "user"} class:mr-8={message.role === "assistant"}>
                <div class="rounded-2xl px-3 py-2 text-sm {message.role === 'user' ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200'}">
                  {message.content}
                </div>

                {#if message.results?.length}
                  <div class="mt-2 space-y-2">
                    {#each message.results as result}
                      <button
                        type="button"
                        class="w-full rounded-xl border border-slate-200 bg-white p-3 text-left transition hover:border-indigo-300 hover:bg-indigo-50 dark:border-slate-700 dark:bg-slate-900 dark:hover:border-indigo-700 dark:hover:bg-slate-800"
                        on:click={() => openAiResult(result.task)}
                      >
                        <div class="flex items-start justify-between gap-3">
                          <div class="min-w-0">
                            <div class="truncate font-medium text-slate-900 dark:text-white">
                              {#if result.task.task_number}#{result.task.task_number} {/if}{result.task.title}
                            </div>
                            <div class="mt-1 text-xs text-slate-500 dark:text-slate-400">
                              {result.task.status} · {result.task.priority || "none"} · score {result.score.toFixed(3)}
                            </div>
                          </div>
                          <span class="shrink-0 rounded-full bg-slate-100 px-2 py-1 text-xs text-slate-500 dark:bg-slate-800 dark:text-slate-400">
                            เปิด
                          </span>
                        </div>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}

            {#if aiChatLoading}
              <div class="mr-8 rounded-2xl bg-slate-100 px-3 py-2 text-sm text-slate-500 dark:bg-slate-800 dark:text-slate-300">
                กำลังค้นหา task...
              </div>
            {/if}

            {#if aiChatError}
              <div class="rounded-2xl bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-950/40 dark:text-red-300">
                {aiChatError}
              </div>
            {/if}
          </div>

          <form class="flex gap-2 border-t border-slate-200 p-3 dark:border-slate-700" on:submit|preventDefault={sendAiChat}>
            <input
              class="min-w-0 flex-1 rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-indigo-500 dark:border-slate-700 dark:bg-slate-950 dark:text-white"
              bind:value={aiChatInput}
              placeholder="ถามเกี่ยวกับ task..."
              disabled={aiChatLoading}
            />
            <button
              type="submit"
              class="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={aiChatLoading || !aiChatInput.trim()}
            >
              ส่ง
            </button>
          </form>
        </div>
      {/if}
    {/if}

    <CommandPalette
      open={$modals.commandPalette}
      tasks={$allTasksIncludingArchived}
      sprints={$sprints}
      projects={$projectList}
      assignees={$assignees}
      t={(key, options) => $_(key, options)}
      toggleTheme={() => theme.toggle()}
      switchView={(v) => viewActions.switchView(v)}
      openTask={(t) => {
        void openTaskPage(t);
      }}
      createTask={() => {
        if (isMyTasksWorkspace) return;
        $editingTask = null;
        uiActions.openModal("form");
        updateUrlTask(null);
      }}
      startTimer={() => {
        /* Not implemented logic */
      }}
      openQuickNotes={() => {
        /* Not implemented logic */
      }}
      applyGlobalSearch={(q) => {
        searchInput.set(q);
        uiActions.closeModal("commandPalette");
      }}
      openBookmarks={() => {
        /* Not implemented logic */
      }}
      openWhiteboard={() => {
        /* Not implemented logic */
      }}
      dailyReflect={() => uiActions.openModal("dailyReflect")}
      openPageSize={() => uiActions.openModal("pageSize")}
      on:close={() => uiActions.closeModal("commandPalette")}
      on:clearFilters={() => {
        filters.set({ ...DEFAULT_FILTERS });
        clearSavedFilters();
      }}
    />
  {/if}
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
