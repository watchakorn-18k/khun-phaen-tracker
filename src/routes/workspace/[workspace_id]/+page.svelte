<script lang="ts">
  import { onMount, onDestroy } from "svelte";
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
  import { Sparkles, Send, X, MessageSquare, ArrowUp, Loader2 } from "lucide-svelte";

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
  let aiGenerateLoading = false;
  let aiChatMode: "search" | "generate" = "search";
  let aiChatMessages: Array<{
    role: "user" | "assistant";
    content: string;
    results?: Array<{ task: Task; score: number }>;
    generatedTask?: any;
    showAssignPrompt?: boolean;
    createdTaskUrl?: string;
    createdTask?: any;
    aiResponse?: string;
  }> = [];
  let aiChatHistory: string[] = [];
  let aiChatHistoryIndex = -1;
  let aiChatMessagesContainer: HTMLDivElement | null = null;

  // Scroll to bottom when messages change (using manual trigger instead of reactive)
  function scrollToBottom() {
    if (aiChatMessagesContainer) {
      requestAnimationFrame(() => {
        if (aiChatMessagesContainer) {
          aiChatMessagesContainer.scrollTop = aiChatMessagesContainer.scrollHeight;
        }
      });
    }
  }
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

    // Handle MongoDB ObjectId format
    let taskId = task.id;
    if (typeof taskId === 'object' && taskId !== null) {
      taskId = (taskId as any).$oid || String(taskId);
    }

    goto(`${base}/workspace/${wsId}/task/${taskId}${roomParam}`);
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

    // Add to history
    aiChatHistory = [...aiChatHistory, message];
    aiChatHistoryIndex = -1;

    aiChatMessages = [...aiChatMessages, { role: "user", content: message }];
    aiChatInput = "";
    aiChatLoading = true;
    aiChatError = "";
    scrollToBottom();

    try {
      const res = await api.data.ai.chatTasks(workspaceId, {
        message,
        limit: 5,
        task_limit: 100,
      });
      const raw = await res.text();
      const data = raw ? JSON.parse(raw) : {};
      if (!res.ok) throw new Error(data.error || `AI chat failed (${res.status})`);
      aiChatMessages = [
        ...aiChatMessages,
        {
          role: "assistant",
          content: data.answer || "เจอ task ที่เกี่ยวข้อง",
          results: data.results || [],
        },
      ];
      scrollToBottom();
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

  async function generateTaskFromAi() {
    const message = aiChatInput.trim();
    const workspaceId = $page.params.workspace_id;
    if (!message || !workspaceId || aiGenerateLoading) return;

    // Add to history
    aiChatHistory = [...aiChatHistory, message];
    aiChatHistoryIndex = -1;

    aiChatMessages = [...aiChatMessages, { role: "user", content: message }];
    aiChatInput = "";
    aiGenerateLoading = true;
    aiChatError = "";
    scrollToBottom();

    // Show loading message
    aiChatMessages = [
      ...aiChatMessages,
      {
        role: "assistant",
        content: "กำลังวิเคราะห์และสร้าง task...",
      },
    ];
    scrollToBottom();

    try {
      const res = await api.data.ai.generateTask(workspaceId, {
        prompt: message,
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || `Task generation failed (${res.status})`);

      // Replace loading message with result
      aiChatMessages = aiChatMessages.slice(0, -1);

      // Extract AI's raw response if available
      const aiRawResponse = data.ai_response || data.raw_response || "";

      aiChatMessages = [
        ...aiChatMessages,
        {
          role: "assistant",
          content: "ฉันสร้าง task ให้แล้ว ต้องการ assign ให้คุณด้วยไหม?",
          generatedTask: data.task,
          showAssignPrompt: true,
          aiResponse: aiRawResponse,
        },
      ];
      scrollToBottom();
    } catch (error) {
      // Remove loading message on error
      aiChatMessages = aiChatMessages.slice(0, -1);
      aiChatError = error instanceof Error ? error.message : "Task generation failed";
    } finally {
      aiGenerateLoading = false;
    }
  }

  async function createTaskFromAi(generatedTask: any, assignToMe: boolean) {
    const workspaceId = $page.params.workspace_id;
    if (!workspaceId) return;

    const today = new Date().toISOString().split("T")[0];

    // Calculate due date: 7 days from today
    const dueDate = new Date();
    dueDate.setDate(dueDate.getDate() + 7);
    const dueDateStr = dueDate.toISOString().split("T")[0];

    try {
      const taskData: any = {
        title: generatedTask.title || "",
        category: generatedTask.category || "feature",
        priority: generatedTask.priority || "medium",
        notes: generatedTask.notes || "",
        project: generatedTask.project || "",
        date: today,
        due_date: dueDateStr,
        duration_minutes: 60,
        status: "todo",
      };

      // Add checklist if AI generated it
      if (generatedTask.checklist && Array.isArray(generatedTask.checklist)) {
        taskData.checklist = generatedTask.checklist.map((text: string, index: number) => ({
          id: `${Date.now()}-${index}`,
          text: text,
          completed: false,
        }));
      }

      if (assignToMe && myAssigneeId) {
        taskData.assignee_ids = [myAssigneeId];
      }

      const res = await api.data.tasks.create(workspaceId, taskData);
      const data = await res.json();

      if (!res.ok) throw new Error(data.error || "Failed to create task");

      // Handle MongoDB ObjectId format
      const task = data.task || data;
      let taskId = task.id || task._id;

      // If id is an object with $oid property, extract it
      if (typeof taskId === 'object' && taskId !== null) {
        taskId = taskId.$oid || String(taskId);
      }

      const taskUrl = `${window.location.origin}${base}/workspace/${workspaceId}/task/${taskId}`;

      aiChatMessages = [
        ...aiChatMessages,
        {
          role: "assistant",
          content: "สร้าง task สำเร็จแล้ว!",
          createdTaskUrl: taskUrl,
          createdTask: task,
        },
      ];

      await loadData();
    } catch (error) {
      aiChatError = error instanceof Error ? error.message : "Failed to create task";
    }
  }

  function cancelTaskCreation() {
    aiChatMessages = [
      ...aiChatMessages,
      {
        role: "assistant",
        content: "ยกเลิกการสร้าง task แล้ว",
      },
    ];
  }

  function handleAiChatKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (aiChatMode === "search") {
        sendAiChat();
      } else {
        generateTaskFromAi();
      }
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (aiChatHistory.length === 0) return;

      if (aiChatHistoryIndex === -1) {
        aiChatHistoryIndex = aiChatHistory.length - 1;
      } else if (aiChatHistoryIndex > 0) {
        aiChatHistoryIndex--;
      }

      aiChatInput = aiChatHistory[aiChatHistoryIndex] || "";
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (aiChatHistoryIndex === -1) return;

      if (aiChatHistoryIndex < aiChatHistory.length - 1) {
        aiChatHistoryIndex++;
        aiChatInput = aiChatHistory[aiChatHistoryIndex] || "";
      } else {
        aiChatHistoryIndex = -1;
        aiChatInput = "";
      }
    }
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
      on:exportVideoWithVoice={(e) => exportActions.handleExportVideoWithVoice(e)}
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
        class="fixed bottom-6 right-6 z-50 flex h-14 w-14 items-center justify-center rounded-full bg-indigo-600 text-white shadow-xl shadow-indigo-500/40 transition-all hover:scale-110 hover:bg-indigo-500 active:scale-95"
        aria-label="เปิด AI task chat"
        on:click={() => (aiChatOpen = !aiChatOpen)}
      >
        <Sparkles size={24} fill={aiChatOpen ? "currentColor" : "none"} />
      </button>

      {#if aiChatOpen}
        <div class="fixed bottom-24 right-6 z-50 flex w-[min(420px,calc(100vw-3rem))] flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-2xl transition-all animate-in fade-in slide-in-from-bottom-4 dark:border-slate-800 dark:bg-slate-900">
          <!-- Header -->
          <div class="flex items-center justify-between border-b border-slate-100 bg-slate-50/50 px-5 py-4 dark:border-slate-800 dark:bg-slate-800/50">
            <div class="flex items-center gap-3">
              <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-100 text-indigo-600 dark:bg-indigo-900/50 dark:text-indigo-400">
                <Sparkles size={18} />
              </div>
              <div>
                <div class="text-sm font-bold text-slate-900 dark:text-white">AI Task Assistant</div>
                <div class="text-[10px] font-medium uppercase tracking-wider text-slate-400 dark:text-slate-500">Semantic Search Engine</div>
              </div>
            </div>
            <button
              type="button"
              class="rounded-full p-1.5 text-slate-400 transition hover:bg-slate-200 hover:text-slate-600 dark:hover:bg-slate-700 dark:hover:text-slate-200"
              on:click={() => (aiChatOpen = false)}
            >
              <X size={18} />
            </button>
          </div>

          <!-- Messages Area -->
          <div bind:this={aiChatMessagesContainer} class="flex max-h-[480px] min-h-[300px] flex-col gap-4 overflow-y-auto p-5 scrollbar-thin">
            {#if aiChatMessages.length === 0}
              <div class="flex flex-col items-center justify-center py-8 text-center">
                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-slate-100 text-slate-400 dark:bg-slate-800 dark:text-slate-600">
                  <MessageSquare size={24} />
                </div>
                <p class="mb-1 text-sm font-medium text-slate-600 dark:text-slate-300">ถามอะไรผมดี?</p>
                <p class="max-w-[200px] text-xs text-slate-400">ลองถามเกี่ยวกับงาน เช่น "งานที่ด่วนที่สุด" หรือ "งานของคุณเอก"</p>
              </div>
            {/if}

            {#each aiChatMessages as message}
              <div class="flex flex-col {message.role === 'user' ? 'items-end' : 'items-start'}">
                <div class="max-w-[85%] rounded-2xl px-4 py-2.5 text-sm shadow-sm {message.role === 'user' ? 'bg-indigo-600 font-medium text-white rounded-tr-none' : 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200 rounded-tl-none border border-slate-200 dark:border-slate-700'}">
                  {message.content}
                </div>

                {#if message.results?.length}
                  <div class="mt-3 w-full space-y-2">
                    <div class="flex items-center gap-2 px-1">
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                      <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400">Relevant Tasks</span>
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                    </div>
                    {#each message.results as result}
                      <button
                        type="button"
                        class="group relative w-full overflow-hidden rounded-2xl border border-slate-100 bg-white p-3 text-left transition-all hover:border-indigo-200 hover:shadow-md dark:border-slate-800 dark:bg-slate-950 dark:hover:border-indigo-900"
                        on:click={() => openAiResult(result.task)}
                      >
                        <div class="flex items-start justify-between gap-3">
                          <div class="min-w-0 flex-1">
                            <div class="flex items-center gap-1.5 truncate">
                              {#if result.task.task_number}
                                <span class="text-[10px] font-black text-indigo-500">#{result.task.task_number}</span>
                              {/if}
                              <span class="truncate text-sm font-semibold text-slate-800 dark:text-slate-100 group-hover:text-indigo-600 dark:group-hover:text-indigo-400">
                                {result.task.title}
                              </span>
                            </div>
                            <div class="mt-1.5 flex flex-wrap items-center gap-2 text-[10px] font-medium text-slate-400">
                              <span class="rounded bg-slate-100 px-1.5 py-0.5 uppercase dark:bg-slate-800">{result.task.status}</span>
                              {#if result.task.priority && result.task.priority !== 'none'}
                                <span class="rounded bg-red-50 px-1.5 py-0.5 text-red-500 dark:bg-red-950/30">{result.task.priority}</span>
                              {/if}
                              <span class="ml-auto text-slate-300 dark:text-slate-600">match {Math.round(result.score * 100)}%</span>
                            </div>
                          </div>
                        </div>
                      </button>
                    {/each}
                  </div>
                {/if}

                {#if message.generatedTask}
                  <div class="mt-3 w-full">
                    <div class="flex items-center gap-2 px-1 mb-2">
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                      <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400">Generated Task</span>
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                    </div>

                    <div class="rounded-2xl border border-emerald-100 bg-emerald-50/50 p-3.5 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                      <div class="mb-2.5 flex items-start justify-between gap-2">
                        <h4 class="text-[13px] font-bold leading-tight text-slate-800 dark:text-slate-100">{message.generatedTask.title}</h4>
                        <div class="flex gap-1 shrink-0">
                          <span class="rounded bg-slate-100 px-1.5 py-0.5 text-[9px] font-medium uppercase text-slate-600 dark:bg-slate-800 dark:text-slate-300">{message.generatedTask.category}</span>
                          <span class="rounded bg-amber-100 px-1.5 py-0.5 text-[9px] font-medium uppercase text-amber-700 dark:bg-amber-900/30 dark:text-amber-400">{message.generatedTask.priority}</span>
                        </div>
                      </div>
                      {#if message.generatedTask.project}
                        <div class="mb-2 text-[11px] text-slate-600 dark:text-slate-400">
                          <span class="font-semibold">Project:</span> {message.generatedTask.project}
                        </div>
                      {/if}
                      {#if message.generatedTask.notes}
                        <div class="mb-2.5 rounded-xl bg-white/60 p-2.5 text-[11px] leading-relaxed text-slate-700 dark:bg-slate-800/40 dark:text-slate-300">
                          <div class="font-semibold mb-1 text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">Description</div>
                          <div class="whitespace-pre-wrap">{message.generatedTask.notes}</div>
                        </div>
                      {/if}

                      {#if message.generatedTask.checklist && message.generatedTask.checklist.length > 0}
                        <div class="mb-2.5 rounded-xl bg-white/60 p-2.5 dark:bg-slate-800/40">
                          <div class="font-semibold mb-1.5 text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">Checklist</div>
                          <div class="space-y-1">
                            {#each message.generatedTask.checklist as item}
                              <div class="flex items-start gap-2 text-[11px] text-slate-700 dark:text-slate-300">
                                <span class="mt-0.5 text-slate-400">☐</span>
                                <span>{item}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/if}

                      {#if message.showAssignPrompt}
                        <div class="flex gap-1.5">
                          <button
                            type="button"
                            class="flex-1 rounded-xl bg-indigo-600 px-3 py-2 text-[11px] font-semibold text-white transition hover:bg-indigo-500"
                            on:click={() => createTaskFromAi(message.generatedTask, true)}
                          >
                            ✓ สร้าง & Assign ให้ฉัน
                          </button>
                          <button
                            type="button"
                            class="flex-1 rounded-xl border border-slate-200 bg-white px-3 py-2 text-[11px] font-semibold text-slate-700 transition hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
                            on:click={() => createTaskFromAi(message.generatedTask, false)}
                          >
                            สร้างโดยไม่ Assign
                          </button>
                          <button
                            type="button"
                            class="rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-[11px] font-semibold text-red-600 transition hover:bg-red-100 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400 dark:hover:bg-red-900/20"
                            on:click={cancelTaskCreation}
                          >
                            ✕
                          </button>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}

                {#if message.createdTaskUrl}
                  <div class="mt-3 w-full">
                    <div class="flex items-center gap-2 px-1 mb-2">
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                      <span class="text-[10px] font-bold uppercase tracking-widest text-emerald-500">Task Created</span>
                      <div class="h-px flex-1 bg-slate-100 dark:bg-slate-800"></div>
                    </div>
                    <a
                      href={message.createdTaskUrl}
                      class="block rounded-2xl border border-emerald-200 bg-gradient-to-br from-emerald-50 to-emerald-100/50 p-4 transition hover:shadow-md dark:border-emerald-800/50 dark:from-emerald-900/20 dark:to-emerald-900/10"
                      on:click={() => aiChatOpen = false}
                    >
                      <div class="mb-2 flex items-center gap-2">
                        <div class="flex h-8 w-8 items-center justify-center rounded-full bg-emerald-500 text-white">
                          <span class="text-lg">✓</span>
                        </div>
                        <div class="flex-1">
                          <div class="text-sm font-bold text-emerald-900 dark:text-emerald-100">
                            {message.createdTask?.title || "Task Created"}
                          </div>
                          {#if message.createdTask?.task_number}
                            <div class="text-xs text-emerald-600 dark:text-emerald-400">
                              #{message.createdTask.task_number}
                            </div>
                          {/if}
                        </div>
                      </div>
                      <div class="text-xs text-emerald-700 dark:text-emerald-300">
                        คลิกเพื่อดู task →
                      </div>
                    </a>
                  </div>
                {/if}
              </div>
            {/each}

            {#if aiChatLoading || aiGenerateLoading}
              <div class="flex items-start gap-2">
                <div class="flex h-8 w-8 animate-pulse items-center justify-center rounded-full bg-slate-100 dark:bg-slate-800">
                  <Sparkles size={14} class="text-slate-400" />
                </div>
                <div class="flex gap-1 py-3">
                  <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-slate-300 dark:bg-slate-600"></div>
                  <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-slate-300 [animation-delay:0.2s] dark:bg-slate-600"></div>
                  <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-slate-300 [animation-delay:0.4s] dark:bg-slate-600"></div>
                </div>
              </div>
            {/if}

            {#if aiChatError}
              <div class="rounded-xl border border-red-100 bg-red-50/50 p-3 text-xs font-medium text-red-600 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400">
                ⚠️ {aiChatError}
              </div>
            {/if}
          </div>

          <!-- Input Area -->
          <div class="border-t border-slate-100 bg-white p-4 dark:border-slate-800 dark:bg-slate-900">
            <div class="mb-3 flex gap-2">
              <button
                type="button"
                class="flex-1 rounded-xl px-3 py-2 text-xs font-semibold transition-all {aiChatMode === 'search' ? 'bg-indigo-600 text-white shadow-sm' : 'bg-slate-100 text-slate-600 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:hover:bg-slate-700'}"
                on:click={() => aiChatMode = 'search'}
              >
                🔍 ค้นหา Task
              </button>
              <button
                type="button"
                class="flex-1 rounded-xl px-3 py-2 text-xs font-semibold transition-all {aiChatMode === 'generate' ? 'bg-emerald-600 text-white shadow-sm' : 'bg-slate-100 text-slate-600 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:hover:bg-slate-700'}"
                on:click={() => aiChatMode = 'generate'}
              >
                ✨ สร้าง Task
              </button>
            </div>
            <form
              class="relative flex items-center"
              on:submit|preventDefault={aiChatMode === 'search' ? sendAiChat : generateTaskFromAi}
            >
              <input
                class="w-full rounded-2xl border border-slate-200 bg-slate-50 py-3 pl-4 pr-12 text-sm text-slate-900 transition-all focus:border-indigo-500 focus:bg-white focus:outline-none focus:ring-4 focus:ring-indigo-500/10 dark:border-slate-700 dark:bg-slate-800/50 dark:text-white dark:focus:border-indigo-500 dark:focus:bg-slate-900"
                bind:value={aiChatInput}
                on:keydown={handleAiChatKeydown}
                placeholder={aiChatMode === 'search' ? 'ค้นหา task เช่น "งานที่ด่วนที่สุด"' : 'สร้าง task เช่น "ทำ API สำหรับ login"'}
                disabled={aiChatLoading || aiGenerateLoading}
              />
              <button
                type="submit"
                class="absolute right-1.5 flex h-9 w-9 items-center justify-center rounded-xl {aiChatMode === 'search' ? 'bg-indigo-600' : 'bg-emerald-600'} text-white transition-all hover:opacity-90 disabled:bg-slate-200 dark:disabled:bg-slate-800"
                disabled={aiChatLoading || aiGenerateLoading || !aiChatInput.trim()}
              >
                <ArrowUp size={20} />
              </button>
            </form>
          </div>
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
