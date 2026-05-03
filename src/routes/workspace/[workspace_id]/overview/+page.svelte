<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { api } from "$lib/apis";
  import { _ } from "svelte-i18n";
  import { createWorkspacePageStore } from "$lib/stores/workspacePageStore";
  import { connectRealtime, disconnectRealtime } from "$lib/stores/realtime";
  import {
    MY_TASKS_WORKSPACE_ID,
    currentWorkspaceOwnerId,
    currentWorkspaceName,
    setWorkspaceId,
  } from "$lib/stores/workspace";
  import { user } from "$lib/stores/auth";
  import { modals, createUIActions } from "$lib/stores/uiActions";
  import StatsPanel from "$lib/components/StatsPanel.svelte";
  import WorkspaceLoading from "$lib/components/WorkspaceLoading.svelte";
  import AccessDenied from "$lib/components/AccessDenied.svelte";
  import {
    LayoutDashboard,
    BarChart3,
    Users,
    Briefcase,
    Clock,
    CheckCircle2,
    AlertCircle,
    ChevronRight,
    FlaskConical,
    Play,
    CircleDashed,
    XCircle,
    Ban,
    FileCheck,
    MinusCircle,
  } from "lucide-svelte";
  import WorkspaceModals from "$lib/components/WorkspaceModals.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import { createKeyboardHandler } from "$lib/stores/keyboardActions";
  import { getKeyboardConfig } from "$lib/stores/workspaceKeyboardConfig";
  import { theme } from "$lib/stores/theme";
  import { sprints } from "$lib/stores/sprintStore";
  import { buildMonthlySummary } from "$lib/utils/monthly-summary";

  const ws = createWorkspacePageStore();
  const {
    workspaceActions,
    loadData,
    editingTask,
    allTasksIncludingArchived,
    taskActions,
    sprintActions,
    viewActions,
    searchInput,
    assignees,
    projectList,
    loadingData,
    workerStats,
    projectStats,
    stats,
    hasAccess,
    checkingAccess,
  } = ws;

  const uiActions = createUIActions();

  const { currentView, pageSize, currentPage } = viewActions;

  const keyboardHandler = createKeyboardHandler(
    getKeyboardConfig({
      uiActions,
      modals,
      editingTask: $editingTask,
      setEditingTask: (v) => ($editingTask = v),
      searchInputRef: null,
      visibleTabs: [],
      viewActions,
    }),
  );

  function cancelEdit() {
    $editingTask = null;
    uiActions.closeModal("form");
  }

  function openTaskPage(task: any) {
    const wsId = $page.params.workspace_id;
    const urlRoom = $page.url.searchParams.get("room");
    const roomParam = urlRoom ? `?room=${urlRoom}` : "";
    goto(`${base}/workspace/${wsId}/task/${task.id}${roomParam}`);
  }

  let activeWorkspaceKey = "";
  let workspaceInitRun = 0;

  $: workspaceId = $page.params.workspace_id;
  $: isOwner = $currentWorkspaceOwnerId === $user?.id;

  async function initializeWorkspace(workspaceId: string, urlRoom: string | null) {
    const runId = ++workspaceInitRun;
    if (!workspaceId || workspaceId === MY_TASKS_WORKSPACE_ID) return;

    checkingAccess.set(true);
    hasAccess.set(false);
    disconnectRealtime();

    if (!urlRoom) return;

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
        loadData();
        loadTestData(workspaceId);
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
    const wsId = $page.params.workspace_id;
    const urlRoom = $page.url.searchParams.get("room");
    const nextWorkspaceKey = `${wsId || ""}:${urlRoom || ""}`;
    if (wsId && nextWorkspaceKey !== activeWorkspaceKey) {
      activeWorkspaceKey = nextWorkspaceKey;
      void initializeWorkspace(wsId, urlRoom);
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

  $: completionRate = $stats.total > 0 ? Math.round(($stats.done / $stats.total) * 100) : 0;

  $: resolvedWorkerStats = ($workerStats || []).map((s: any) => {
    const assignee = ($assignees || []).find((a: any) => String(a.id) === String(s.id));
    return {
      name: assignee ? assignee.name : s.id,
      count: s.taskCount
    };
  }).sort((a, b) => b.count - a.count);

  $: resolvedProjectStats = ($projectStats || []).map((s: any) => {
    const project = ($projectList || []).find((p: any) => String(p.id) === String(s.id));
    return {
      name: project ? project.name : (s.id === "none" ? $_("page__unassigned_project") : s.id),
      count: s.taskCount
    };
  }).sort((a, b) => b.count - a.count);

  $: monthlySummary = buildMonthlySummary($allTasksIncludingArchived);

  type TestCaseCounts = {
    total: number;
    draft: number;
    actual: number;
    passed: number;
    failed: number;
    blocked: number;
    deprecated: number;
  };

  type TestRunSummary = {
    _id: string;
    name: string;
    status: "running" | "completed" | "aborted";
    created_at: string;
    stats?: {
      total: number;
      passed: number;
      failed: number;
      pending: number;
    };
  };

  let testCaseCounts: TestCaseCounts = { total: 0, draft: 0, actual: 0, passed: 0, failed: 0, blocked: 0, deprecated: 0 };
  let recentTestRuns: TestRunSummary[] = [];
  let loadingTestData = false;

  async function loadTestData(wsId: string) {
    loadingTestData = true;
    try {
      const [countsRes, runsRes] = await Promise.all([
        api.data.testCases.counts(wsId),
        api.testRuns.list(wsId, 1, 5),
      ]);
      if (countsRes.ok) {
        const d = await countsRes.json();
        testCaseCounts = d.counts ?? d;
      }
      if (runsRes.ok) {
        const d = await runsRes.json();
        recentTestRuns = d.data ?? d.test_runs ?? d ?? [];
      }
    } catch (_) {
      // ignore
    } finally {
      loadingTestData = false;
    }
  }

  $: tcPassRate = testCaseCounts.total > 0 ? Math.round((testCaseCounts.passed / testCaseCounts.total) * 100) : 0;
</script>

<svelte:head>
  <title>{$_("meta__overview_title", { values: { name: $currentWorkspaceName || "Khun Phaen" } })}</title>
  <meta name="description" content={$_("meta__overview_desc", { values: { name: $currentWorkspaceName || "Khun Phaen" } })} />
</svelte:head>

<div class="px-4 sm:px-6 space-y-8 pb-24 pt-6">
  {#if $checkingAccess}
    <WorkspaceLoading />
  {:else if !$hasAccess}
    <AccessDenied />
  {:else}
    <!-- Header Section -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-black text-gray-900 dark:text-white tracking-tight flex items-center gap-2">
          <LayoutDashboard class="text-primary" size={28} />
          {$_("sidebar__overview")}
        </h1>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 font-medium">
          {$_("overview__welcome_subtitle")}
        </p>
      </div>
      
      <div class="flex items-center gap-2">
        <div class="px-4 py-2 bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></div>
          <span class="text-xs font-bold text-gray-600 dark:text-gray-300 uppercase tracking-widest">
            Live Updates Enabled
          </span>
        </div>
      </div>
    </div>

    <!-- Stats Panel -->
    <StatsPanel stats={$stats} {isOwner} />

    <!-- Progress Dashboard -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Project Progress Chart/Card -->
      <div class="lg:col-span-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-[32px] p-6 shadow-sm overflow-hidden relative group">
        <div class="absolute top-0 right-0 p-8 opacity-[0.03] dark:opacity-[0.05] group-hover:opacity-[0.07] transition-opacity">
          <BarChart3 size={160} />
        </div>
        
        <div class="relative z-10">
          <div class="flex items-center justify-between mb-8">
            <h2 class="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2">
              <BarChart3 class="text-blue-500" size={20} />
              {$_("overview__project_status")}
            </h2>
          </div>

          <div class="space-y-6">
            <!-- Overall Progress -->
            <div class="space-y-2">
              <div class="flex justify-between items-end">
                <span class="text-sm font-bold text-gray-500 dark:text-gray-400 uppercase tracking-widest">{$_("overview__overall_completion")}</span>
                <span class="text-3xl font-black text-primary tracking-tighter">{completionRate}%</span>
              </div>
              <div class="h-4 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden p-1">
                <div 
                  class="h-full bg-gradient-to-r from-primary to-blue-400 rounded-full transition-all duration-1000 ease-out"
                  style="width: {completionRate}%"
                ></div>
              </div>
            </div>

            <!-- Detailed Stats Grid -->
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 pt-4">
              <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50">
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1">{$_("statsPanel__in_progress")}</p>
                <p class="text-xl font-black text-blue-500">{$stats.in_progress}</p>
              </div>
              <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50">
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1">{$_("statsPanel__in_test")}</p>
                <p class="text-xl font-black text-purple-500">{$stats.in_test}</p>
              </div>
              <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50">
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1">{$_("overview__overdue")}</p>
                <p class="text-xl font-black text-rose-500">0</p> <!-- Placeholder for now -->
              </div>
              <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50">
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1">{$_("statsPanel__done")}</p>
                <p class="text-xl font-black text-emerald-500">{$stats.done}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Actions / Team Card -->
      <div class="bg-gradient-to-br from-gray-900 to-slate-800 dark:from-gray-800 dark:to-slate-900 rounded-[32px] p-6 text-white shadow-xl flex flex-col justify-between overflow-hidden relative group">
        <div class="absolute -bottom-10 -right-10 opacity-10 group-hover:scale-110 transition-transform duration-700">
          <Users size={200} />
        </div>
        
        <div class="relative z-10">
          <h2 class="text-lg font-black flex items-center gap-2 mb-6">
            <Users size={20} class="text-blue-400" />
            {$_("overview__team_activity")}
          </h2>

          <div class="space-y-4">
            {#each resolvedWorkerStats.slice(0, 4) as worker}
              <div class="flex items-center justify-between p-3 bg-white/5 hover:bg-white/10 rounded-2xl transition-colors border border-white/5">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center font-black text-xs uppercase">
                    {worker.name.charAt(0)}
                  </div>
                  <span class="text-sm font-bold truncate max-w-[120px]">{worker.name}</span>
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-xs font-black text-gray-400">{worker.count} {$_("overview__tasks")}</span>
                  <div class="w-1.5 h-1.5 rounded-full bg-emerald-400"></div>
                </div>
              </div>
            {/each}
            
            {#if resolvedWorkerStats.length === 0}
              <div class="py-8 text-center opacity-40">
                <Users size={32} class="mx-auto mb-2" />
                <p class="text-xs font-bold uppercase tracking-widest">No team activity</p>
              </div>
            {/if}
          </div>
        </div>

        <button
          on:click={() => goto(`${base}/workspace/${workspaceId}${$page.url.search}`)}
          class="relative z-10 mt-6 w-full py-3 bg-white/10 hover:bg-white/20 text-white border border-white/20 rounded-2xl font-black text-xs uppercase tracking-widest flex items-center justify-center gap-2 transition-colors"
        >
          {$_("overview__view_all_issues")}
          <ChevronRight size={14} />
        </button>
      </div>
    </div>

    <!-- Bottom Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- Projects Overview -->
      <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-[32px] p-6 shadow-sm">
        <h2 class="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2 mb-6">
          <Briefcase class="text-indigo-500" size={20} />
          {$_("overview__active_projects")}
        </h2>
        
        <div class="space-y-4">
          {#each resolvedProjectStats as project}
            <div class="group">
              <div class="flex justify-between items-center mb-1.5 px-1">
                <span class="text-sm font-bold text-gray-700 dark:text-gray-300">{project.name}</span>
                <span class="text-xs font-black text-gray-400">{project.count} tasks</span>
              </div>
              <div class="h-2 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden">
                <div 
                  class="h-full bg-indigo-500 rounded-full group-hover:scale-x-105 transition-transform origin-left"
                  style="width: {Math.min(100, (project.count / ($stats.total || 1)) * 100)}%"
                ></div>
              </div>
            </div>
          {:else}
            <div class="py-12 text-center text-gray-400">
              <Briefcase size={40} class="mx-auto mb-3 opacity-20" />
              <p class="text-sm font-bold uppercase tracking-widest opacity-40">No active projects</p>
            </div>
          {/each}
        </div>
      </div>

      <!-- Recent Updates / Health -->
      <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-[32px] p-6 shadow-sm">
        <h2 class="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2 mb-6">
          <Clock class="text-emerald-500" size={20} />
          {$_("overview__project_health")}
        </h2>

        <div class="grid grid-cols-1 gap-4">
          <div class="flex items-center gap-4 p-4 bg-emerald-500/5 border border-emerald-500/10 rounded-2xl">
            <div class="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center text-emerald-600 dark:text-emerald-400">
              <CheckCircle2 size={20} />
            </div>
            <div>
              <p class="text-xs font-black text-gray-400 uppercase tracking-widest mb-0.5">Reliability</p>
              <p class="text-sm font-bold text-gray-900 dark:text-white">Stable Performance</p>
            </div>
          </div>

          <div class="flex items-center gap-4 p-4 bg-amber-500/5 border border-amber-500/10 rounded-2xl opacity-60">
            <div class="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center text-amber-600 dark:text-amber-400">
              <Clock size={20} />
            </div>
            <div>
              <p class="text-xs font-black text-gray-400 uppercase tracking-widest mb-0.5">Deadline Velocity</p>
              <p class="text-sm font-bold text-gray-900 dark:text-white">Maintaining Pace</p>
            </div>
          </div>

          <div class="flex items-center gap-4 p-4 bg-blue-500/5 border border-blue-500/10 rounded-2xl opacity-60">
            <div class="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center text-blue-600 dark:text-blue-400">
              <AlertCircle size={20} />
            </div>
            <div>
              <p class="text-xs font-black text-gray-400 uppercase tracking-widest mb-0.5">Resource Load</p>
              <p class="text-sm font-bold text-gray-900 dark:text-white">Optimized Allocation</p>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Test Cases & Test Runs Section -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Test Case Stats -->
      <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-[32px] p-6 shadow-sm overflow-hidden relative group">
        <div class="absolute top-0 right-0 p-8 opacity-[0.03] dark:opacity-[0.05] group-hover:opacity-[0.07] transition-opacity">
          <FlaskConical size={140} />
        </div>
        <div class="relative z-10">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2">
              <FlaskConical class="text-violet-500" size={20} />
              Test Cases
            </h2>
            <button
              on:click={() => goto(`${base}/workspace/${workspaceId}/test-cases${$page.url.search}`)}
              class="text-xs font-bold text-gray-400 hover:text-primary flex items-center gap-1 transition-colors"
            >
              ดูทั้งหมด <ChevronRight size={14} />
            </button>
          </div>

          {#if loadingTestData}
            <div class="py-8 text-center text-gray-400 text-sm font-bold">กำลังโหลด...</div>
          {:else}
            <!-- Pass Rate Bar -->
            <div class="space-y-2 mb-6">
              <div class="flex justify-between items-end">
                <span class="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-widest">Pass Rate</span>
                <span class="text-2xl font-black text-violet-500 tracking-tighter">{tcPassRate}%</span>
              </div>
              <div class="h-3 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden p-0.5">
                <div
                  class="h-full bg-gradient-to-r from-violet-500 to-purple-400 rounded-full transition-all duration-1000 ease-out"
                  style="width: {tcPassRate}%"
                ></div>
              </div>
            </div>

            <!-- Status Grid -->
            <div class="grid grid-cols-3 gap-3">
              <div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50 text-center">
                <div class="flex items-center justify-center mb-1"><CircleDashed size={14} class="text-gray-400" /></div>
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-0.5">Draft</p>
                <p class="text-lg font-black text-gray-600 dark:text-gray-300">{testCaseCounts.draft}</p>
              </div>
              <div class="p-3 bg-blue-50 dark:bg-blue-900/10 rounded-2xl border border-blue-100 dark:border-blue-800/30 text-center">
                <div class="flex items-center justify-center mb-1"><FileCheck size={14} class="text-blue-500" /></div>
                <p class="text-[10px] font-black text-blue-400 uppercase tracking-widest mb-0.5">Actual</p>
                <p class="text-lg font-black text-blue-600 dark:text-blue-400">{testCaseCounts.actual}</p>
              </div>
              <div class="p-3 bg-emerald-50 dark:bg-emerald-900/10 rounded-2xl border border-emerald-100 dark:border-emerald-800/30 text-center">
                <div class="flex items-center justify-center mb-1"><CheckCircle2 size={14} class="text-emerald-500" /></div>
                <p class="text-[10px] font-black text-emerald-400 uppercase tracking-widest mb-0.5">Passed</p>
                <p class="text-lg font-black text-emerald-600 dark:text-emerald-400">{testCaseCounts.passed}</p>
              </div>
              <div class="p-3 bg-rose-50 dark:bg-rose-900/10 rounded-2xl border border-rose-100 dark:border-rose-800/30 text-center">
                <div class="flex items-center justify-center mb-1"><XCircle size={14} class="text-rose-500" /></div>
                <p class="text-[10px] font-black text-rose-400 uppercase tracking-widest mb-0.5">Failed</p>
                <p class="text-lg font-black text-rose-600 dark:text-rose-400">{testCaseCounts.failed}</p>
              </div>
              <div class="p-3 bg-amber-50 dark:bg-amber-900/10 rounded-2xl border border-amber-100 dark:border-amber-800/30 text-center">
                <div class="flex items-center justify-center mb-1"><Ban size={14} class="text-amber-500" /></div>
                <p class="text-[10px] font-black text-amber-400 uppercase tracking-widest mb-0.5">Blocked</p>
                <p class="text-lg font-black text-amber-600 dark:text-amber-400">{testCaseCounts.blocked}</p>
              </div>
              <div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-2xl border border-gray-100 dark:border-gray-700/50 text-center">
                <div class="flex items-center justify-center mb-1"><MinusCircle size={14} class="text-gray-400" /></div>
                <p class="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-0.5">Deprecated</p>
                <p class="text-lg font-black text-gray-500 dark:text-gray-400">{testCaseCounts.deprecated}</p>
              </div>
            </div>

            <div class="mt-4 pt-4 border-t border-gray-100 dark:border-gray-800 flex justify-between items-center">
              <span class="text-xs font-bold text-gray-400 uppercase tracking-widest">Total</span>
              <span class="text-xl font-black text-gray-700 dark:text-gray-200">{testCaseCounts.total}</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Recent Test Runs -->
      <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-[32px] p-6 shadow-sm overflow-hidden relative group">
        <div class="absolute top-0 right-0 p-8 opacity-[0.03] dark:opacity-[0.05] group-hover:opacity-[0.07] transition-opacity">
          <Play size={140} />
        </div>
        <div class="relative z-10">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2">
              <Play class="text-indigo-500" size={20} />
              Test Runs
            </h2>
            <button
              on:click={() => goto(`${base}/workspace/${workspaceId}/test-cases${$page.url.search}`)}
              class="text-xs font-bold text-gray-400 hover:text-primary flex items-center gap-1 transition-colors"
            >
              ดูทั้งหมด <ChevronRight size={14} />
            </button>
          </div>

          {#if loadingTestData}
            <div class="py-8 text-center text-gray-400 text-sm font-bold">กำลังโหลด...</div>
          {:else if recentTestRuns.length === 0}
            <div class="py-12 text-center text-gray-400">
              <Play size={40} class="mx-auto mb-3 opacity-20" />
              <p class="text-sm font-bold uppercase tracking-widest opacity-40">ยังไม่มี Test Run</p>
            </div>
          {:else}
            <div class="space-y-3">
              {#each recentTestRuns as run}
                <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800/50 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700/50 transition-colors">
                  <div class="flex items-center gap-3 min-w-0">
                    <div class="w-2 h-2 rounded-full flex-shrink-0
                      {run.status === 'completed' ? 'bg-emerald-500' :
                       run.status === 'aborted' ? 'bg-rose-500' : 'bg-blue-500 animate-pulse'}">
                    </div>
                    <span class="text-sm font-bold text-gray-800 dark:text-gray-200 truncate">{run.name}</span>
                  </div>
                  <div class="flex items-center gap-2 flex-shrink-0 ml-2">
                    {#if run.stats}
                      <span class="text-xs font-black text-emerald-500">{run.stats.passed}P</span>
                      <span class="text-xs font-black text-rose-500">{run.stats.failed}F</span>
                    {/if}
                    <span class="text-[10px] font-black uppercase tracking-widest px-2 py-0.5 rounded-lg
                      {run.status === 'completed' ? 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400' :
                       run.status === 'aborted' ? 'bg-rose-100 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400' :
                       'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'}">
                      {run.status}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <WorkspaceModals
      modals={$modals}
      editingTask={$editingTask}
      assignees={$assignees}
      {isOwner}
      projectList={$projectList}
      sprints={$sprints}
      allTasksIncludingArchived={$allTasksIncludingArchived}
      qrExportTasks={[]}
      loadingData={$loadingData}
      workerStats={$workerStats}
      projectStats={$projectStats}
      {monthlySummary}
      monthlySummaryRef={null}
      workspaceId={String($page.params.workspace_id || "")}
      newPageSize={20}
      on:addTask={(e) => taskActions.handleAddTask(e)}
      on:cancelEdit={cancelEdit}
      on:addAssignee={(e) => taskActions.handleAddAssignee(e)}
      on:checklistUpdate={(e) => taskActions.handleChecklistUpdate(e)}
      on:closeModal={(e) => uiActions.closeModal(e.detail)}
      on:closeWorkerManager={() => uiActions.closeModal("workerManager")}
      on:addWorker={(e) => workspaceActions.handleAddWorker(e)}
      on:updateWorker={(e) => workspaceActions.handleUpdateWorker(e)}
      on:deleteWorker={(e) => workspaceActions.handleDeleteWorker(e)}
      on:closeSprintManager={() => uiActions.closeModal("sprintManager")}
      on:completeSprint={(e) => sprintActions.handleCompleteSprint(e)}
      on:deleteSprint={(e) => sprintActions.handleDeleteSprint(e)}
      on:moveTasksToSprint={(e) => sprintActions.handleMoveTasksToSprint(e)}
      on:closeProjectManager={() => uiActions.closeModal("projectManager")}
      on:addProject={(e) => workspaceActions.handleAddProject(e)}
      on:updateProject={(e) => workspaceActions.handleUpdateProject(e)}
      on:deleteProject={(e) => workspaceActions.handleDeleteProject(e)}
    />

    <CommandPalette
      open={$modals.commandPalette}
      tasks={$allTasksIncludingArchived}
      sprints={$sprints}
      projects={$projectList}
      assignees={$assignees}
      t={(key, options) => $_(key, options)}
      toggleTheme={() => theme.toggle()}
      switchView={(v) => {
        if (v === 'overview') return;
        goto(`${base}/workspace/${workspaceId}${$page.url.search}`);
      }}
      openTask={(t) => {
        void openTaskPage(t);
      }}
      createTask={() => {
        $editingTask = null;
        uiActions.openModal("form");
      }}
      startTimer={() => {}}
      openQuickNotes={() => {}}
      applyGlobalSearch={(q) => {
        searchInput.set(q);
        uiActions.closeModal("commandPalette");
        goto(`${base}/workspace/${workspaceId}${$page.url.search}`);
      }}
      openBookmarks={() => {}}
      openWhiteboard={() => {}}
      on:close={() => uiActions.closeModal("commandPalette")}
      dailyReflect={() => uiActions.openModal("dailyReflect")}
      openPageSize={() => uiActions.openModal("pageSize")}
    />
  {/if}
</div>

<style>
  :global(body) {
    background-attachment: fixed;
  }
</style>
