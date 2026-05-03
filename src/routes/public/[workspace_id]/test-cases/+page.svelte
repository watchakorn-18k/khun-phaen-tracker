<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { api } from "$lib/apis";
  import { theme } from "$lib/stores/theme";
  import {
    Search,
    ChevronDown,
    ChevronRight,
    Moon,
    Sun,
    FileDown,
    Activity,
    CheckCircle2,
    XCircle,
    MinusCircle,
    Clock,
    SkipForward,
    AlertOctagon,
    ChevronUp,
    Layers,
  } from "lucide-svelte";
  import favicon from "$lib/assets/favicon.svg";

  type TestCase = {
    id: string;
    test_no: number;
    title: string;
    priority: "high" | "medium" | "low";
    status: string;
    description: string;
    assign_dev: string;
    fixed: string;
    suite_id: string;
    preconditions?: string;
    postconditions?: string;
    input?: string;
    expected_result?: string;
    actual_result?: string;
    step_format?: string;
    gherkin_steps?: any[];
    classic_steps?: any[];
  };

  type Suite = {
    id: string;
    title: string;
    case_count: number;
    cases: TestCase[];
    collapsed: boolean;
    page: number;
    hasMore: boolean;
  };

  type LatestRunStats = {
    total: number;
    pending: number;
    passed: number;
    failed: number;
    blocked: number;
    skipped: number;
    invalid: number;
  };

  type LatestTestRun = {
    id: string;
    name: string;
    description: string | null;
    default_assignee: string;
    operating_system: string;
    status: string;
    stats: LatestRunStats;
    test_cases: { test_case_id: string; status: string }[];
    created_at: string | null;
    updated_at: string | null;
  };

  $: workspaceId = $page.params.workspace_id;

  let suites: Suite[] = [];
  let loading = true;
  let exporting = false;
  let query = "";
  let searchTimeout: any;
  let filterPriority = "";
  let filterStatus = "";

  let latestRun: LatestTestRun | null = null;
  let runCaseMap = new Map<string, string>();
  let showRunBanner = true;

  function mapCase(tc: any): TestCase {
    return {
      id: tc.id || tc._id,
      test_no: tc.test_no,
      title: tc.name || tc.title || "",
      priority: tc.priority || "medium",
      status: tc.status || "actual",
      description: tc.description || "",
      assign_dev: tc.assign_dev || "unassigned",
      fixed: tc.fixed || "no",
      suite_id: tc.suite_id || "",
      preconditions: tc.preconditions,
      postconditions: tc.postconditions,
      input: tc.input,
      expected_result: tc.expected_result,
      actual_result: tc.actual_result,
      step_format: tc.step_format,
      gherkin_steps: tc.gherkin_steps,
      classic_steps: tc.classic_steps,
    };
  }

  async function loadData() {
    if (!workspaceId) return;
    loading = true;
    try {
      const [suiteResp, runResp] = await Promise.all([
        api.public.listSuites(workspaceId),
        api.public.getLatestTestRun(workspaceId),
      ]);

      if (!suiteResp.ok) return;
      const suiteData = await suiteResp.json();

      if (runResp.ok) {
        const runData: LatestTestRun = await runResp.json();
        latestRun = runData;
        runCaseMap = new Map(
          runData.test_cases.map((tc) => [tc.test_case_id, tc.status]),
        );
      }

      const suiteList = suiteData.map((s: any) => ({
        id: s.id || s._id,
        title: s.title,
        case_count: s.case_count ?? 0,
        collapsed: false,
        page: 1,
        hasMore: false,
        cases: [] as TestCase[],
      }));

      const params: Record<string, any> = { limit: 50 };
      if (query) params.q = query;
      if (filterPriority) params.priority = filterPriority;
      if (filterStatus) params.status = filterStatus;

      const casesPromises = suiteList.map((s: any) =>
        api.public.listTestCases(workspaceId, {
          suite_id: s.id === "unassigned" ? "none" : s.id,
          ...params,
        }),
      );

      const casesResponses = await Promise.all(casesPromises);
      for (let i = 0; i < suiteList.length; i++) {
        if (casesResponses[i].ok) {
          const data = await casesResponses[i].json();
          suiteList[i].cases = data.map(mapCase);
          suiteList[i].hasMore = data.length >= 50;
        }
      }

      suites = suiteList;
    } catch (e) {
      console.error("Failed to load public data:", e);
    } finally {
      loading = false;
    }
  }

  async function exportToCSV() {
    if (!workspaceId || exporting) return;
    exporting = true;
    try {
      const casesResp = await api.public.getAllTestCases(workspaceId);
      if (!casesResp.ok) return;
      const allCasesData = await casesResp.json();
      const allCases = allCasesData.map(mapCase);

      const suiteMap = new Map<string, string>(
        suites.map((s) => [s.id, s.title]),
      );
      suiteMap.set("unassigned", "Unassigned");

      const headers = [
        "Suite",
        "Test Case No",
        "Test Name",
        "Status",
        "Run Result",
        "Assign Dev",
        "Precondition",
        "Input",
        "Expected Result",
        "Actual Result",
        "Test Step",
      ];

      let rows: string[][] = [];
      allCases.forEach((c: TestCase) => {
        let steps = "";
        if (c.step_format === "gherkin") {
          steps = (c.gherkin_steps || [])
            .map((s: any) => `${s.keyword} ${s.text}`)
            .join("\n");
        } else {
          steps = (c.classic_steps || []).map((s: any) => s.action).join("\n");
        }
        rows.push([
          String(suiteMap.get(c.suite_id) || "Unassigned"),
          String(c.test_no),
          String(c.title || ""),
          String(c.status || ""),
          String(runCaseMap.get(c.id) || "—"),
          String(c.assign_dev || "unassigned"),
          String(c.preconditions || ""),
          String(c.input || ""),
          String(c.expected_result || ""),
          String(c.actual_result || ""),
          String(steps || ""),
        ]);
      });

      let csv = headers.join(",") + "\n";
      rows.forEach((row) => {
        csv +=
          row.map((v) => `"${(v || "").replace(/"/g, '""')}"`).join(",") + "\n";
      });

      const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
      const link = document.createElement("a");
      const url = URL.createObjectURL(blob);
      link.setAttribute("href", url);
      link.setAttribute("download", `test_cases_${workspaceId}.csv`);
      link.style.visibility = "hidden";
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    } catch (e) {
      console.error("Failed to export:", e);
    } finally {
      exporting = false;
    }
  }

  function handleSearch() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => loadData(), 300);
  }

  $: totalCases = suites.reduce((t, s) => t + (s.case_count || 0), 0);
  $: totalSuites = suites.filter((s) => s.id !== "unassigned").length;

  $: runPassPct =
    latestRun && latestRun.stats.total > 0
      ? Math.round((latestRun.stats.passed / latestRun.stats.total) * 100)
      : 0;
  $: runFailPct =
    latestRun && latestRun.stats.total > 0
      ? Math.round((latestRun.stats.failed / latestRun.stats.total) * 100)
      : 0;
  $: runBlockPct =
    latestRun && latestRun.stats.total > 0
      ? Math.round((latestRun.stats.blocked / latestRun.stats.total) * 100)
      : 0;
  $: runPendingPct =
    latestRun && latestRun.stats.total > 0
      ? Math.max(0, 100 - runPassPct - runFailPct - runBlockPct)
      : 0;

  function formatDate(iso: string | null): string {
    if (!iso) return "—";
    try {
      return new Intl.DateTimeFormat("en-GB", {
        day: "2-digit",
        month: "short",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  }

  function priorityBadge(p: string) {
    switch (p) {
      case "high":
        return "bg-rose-100 text-rose-700 border-rose-200 dark:bg-rose-900/30 dark:text-rose-300 dark:border-rose-800";
      case "medium":
        return "bg-amber-100 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-300 dark:border-amber-800";
      case "low":
        return "bg-emerald-100 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-300 dark:border-emerald-800";
      default:
        return "bg-slate-100 text-slate-600 border-slate-200 dark:bg-gray-800 dark:text-gray-400 dark:border-gray-700";
    }
  }

  function statusBadge(s: string) {
    switch (s) {
      case "passed":
        return "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300";
      case "failed":
        return "bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-300";
      case "blocked":
        return "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300";
      case "deprecated":
        return "bg-slate-100 text-slate-500 dark:bg-gray-800 dark:text-gray-500";
      case "draft":
        return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300";
      default:
        return "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300";
    }
  }

  function fixedBadge(f: string) {
    return f === "yes"
      ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300"
      : "bg-slate-100 text-slate-500 dark:bg-gray-800 dark:text-gray-400";
  }

  function runResultBadge(s: string) {
    switch (s) {
      case "passed":
        return "bg-emerald-100 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-300 dark:border-emerald-800";
      case "failed":
        return "bg-rose-100 text-rose-700 border-rose-200 dark:bg-rose-900/30 dark:text-rose-300 dark:border-rose-800";
      case "blocked":
        return "bg-orange-100 text-orange-700 border-orange-200 dark:bg-orange-900/30 dark:text-orange-300 dark:border-orange-800";
      case "skipped":
        return "bg-slate-100 text-slate-500 border-slate-200 dark:bg-gray-800 dark:text-gray-400 dark:border-gray-700";
      case "invalid":
        return "bg-red-100 text-red-700 border-red-200 dark:bg-red-900/30 dark:text-red-300 dark:border-red-800";
      default:
        return "bg-indigo-50 text-indigo-500 border-indigo-100 dark:bg-indigo-900/20 dark:text-indigo-400 dark:border-indigo-800";
    }
  }

  function runStatusBadge(s: string) {
    switch (s) {
      case "completed":
        return "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300";
      case "aborted":
        return "bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-300";
      default:
        return "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300";
    }
  }

  function toggleSuite(id: string) {
    suites = suites.map((s) =>
      s.id === id ? { ...s, collapsed: !s.collapsed } : s,
    );
  }

  onMount(() => {
    loadData();
  });
</script>

<svelte:head>
  <title>Test Cases — Public View</title>
  <meta
    name="description"
    content="Public view of test cases for this workspace."
  />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div
  class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-indigo-50/30 dark:from-gray-950 dark:via-gray-950 dark:to-indigo-950/10"
  style="font-family: 'Inter', sans-serif;"
>
  <!-- Top Nav -->
  <header
    class="sticky top-0 z-30 border-b border-slate-200/80 bg-white/80 backdrop-blur-xl dark:border-gray-800/80 dark:bg-gray-950/80"
  >
    <div class="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
      <div class="flex items-center gap-3">
        <img src={favicon} alt="logo" class="h-8 w-8" />
        <div>
          <h1 class="text-lg font-black text-slate-800 dark:text-white">
            Test Case Repository
          </h1>
          <p class="text-xs font-medium text-slate-500 dark:text-gray-400">
            {totalCases} cases · {totalSuites} suites
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="flex items-center gap-2 h-9 px-4 rounded-lg bg-indigo-600 text-white text-sm font-bold transition-all hover:bg-indigo-700 hover:shadow-lg hover:shadow-indigo-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
          on:click={exportToCSV}
          disabled={exporting}
        >
          {#if exporting}
            <div
              class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
            ></div>
            Exporting...
          {:else}
            <FileDown size={16} />
            Export CSV
          {/if}
        </button>

        <button
          class="grid h-9 w-9 place-items-center rounded-lg border border-slate-200 text-slate-500 transition-colors hover:bg-slate-50 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => theme.toggle()}
          title="Toggle theme"
        >
          {#if $theme === "dark"}
            <Sun size={16} />
          {:else}
            <Moon size={16} />
          {/if}
        </button>
      </div>
    </div>
  </header>

  <main class="mx-auto max-w-7xl px-6 py-8">
    <!-- Latest Test Run Banner -->
    {#if latestRun}
      <div
        class="mb-8 overflow-hidden rounded-2xl border border-indigo-200/60 bg-gradient-to-br from-indigo-50 via-white to-purple-50/40 shadow-sm dark:border-indigo-800/40 dark:from-indigo-950/40 dark:via-gray-900 dark:to-purple-950/20"
      >
        <!-- Banner Header -->
        <div class="flex items-center justify-between px-6 pt-5 pb-0">
          <div class="flex items-center gap-3">
            <div
              class="flex h-9 w-9 items-center justify-center rounded-xl bg-indigo-600/10 dark:bg-indigo-500/20"
            >
              <Activity
                size={18}
                class="text-indigo-600 dark:text-indigo-400"
              />
            </div>
            <div>
              <div class="flex items-center gap-2 flex-wrap">
                <span
                  class="text-xs font-black uppercase tracking-widest text-indigo-500 dark:text-indigo-400"
                >
                  Latest Test Run
                </span>
                <span
                  class="inline-block rounded-full px-2.5 py-0.5 text-[10px] font-black uppercase {runStatusBadge(
                    latestRun.status,
                  )}"
                >
                  {latestRun.status}
                </span>
              </div>
              <div class="flex items-center gap-2 mt-0.5 flex-wrap">
                <h2 class="text-base font-black text-slate-800 dark:text-white">
                  {latestRun.name}
                </h2>
                {#if latestRun.operating_system}
                  <span
                    class="rounded-md border border-slate-200 bg-white px-2 py-0.5 text-[10px] font-bold text-slate-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400"
                  >
                    {latestRun.operating_system}
                  </span>
                {/if}
              </div>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <span
              class="hidden sm:block text-xs font-medium text-slate-400 dark:text-gray-500"
            >
              {formatDate(latestRun.created_at)}
            </span>
            <button
              class="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white/80 px-3 py-1.5 text-xs font-bold text-slate-500 transition-colors hover:bg-slate-50 dark:border-gray-700 dark:bg-gray-800/80 dark:text-gray-400 dark:hover:bg-gray-700"
              on:click={() => (showRunBanner = !showRunBanner)}
            >
              {#if showRunBanner}
                <ChevronUp size={12} />
                Hide
              {:else}
                <ChevronDown size={12} />
                Show
              {/if}
            </button>
          </div>
        </div>

        {#if showRunBanner}
          <div class="px-6 pt-4 pb-5 space-y-4">
            <!-- Progress Bar -->
            <div>
              <div class="mb-1.5 flex items-center justify-between">
                <span
                  class="text-xs font-bold text-slate-500 dark:text-gray-400"
                >
                  Progress — {latestRun.stats.passed}/{latestRun.stats.total} passed
                </span>
                <span
                  class="text-xs font-black text-indigo-600 dark:text-indigo-400"
                >
                  {runPassPct}%
                </span>
              </div>
              <div
                class="flex h-3 w-full overflow-hidden rounded-full bg-slate-100 dark:bg-gray-800"
              >
                {#if latestRun.stats.passed > 0}
                  <div
                    class="h-full bg-emerald-500 transition-all duration-500"
                    style="width: {runPassPct}%"
                  ></div>
                {/if}
                {#if latestRun.stats.failed > 0}
                  <div
                    class="h-full bg-rose-500 transition-all duration-500"
                    style="width: {runFailPct}%"
                  ></div>
                {/if}
                {#if latestRun.stats.blocked > 0}
                  <div
                    class="h-full bg-orange-400 transition-all duration-500"
                    style="width: {runBlockPct}%"
                  ></div>
                {/if}
                {#if runPendingPct > 0}
                  <div
                    class="h-full bg-slate-200 dark:bg-gray-700 transition-all duration-500"
                    style="width: {runPendingPct}%"
                  ></div>
                {/if}
              </div>
            </div>

            <!-- Stats Chips -->
            <div class="flex flex-wrap gap-2">
              <div
                class="flex items-center gap-1.5 rounded-xl border border-emerald-200 bg-emerald-50 px-3 py-1.5 dark:border-emerald-800/50 dark:bg-emerald-900/20"
              >
                <CheckCircle2
                  size={13}
                  class="text-emerald-600 dark:text-emerald-400"
                />
                <span
                  class="text-xs font-black text-emerald-700 dark:text-emerald-300"
                >
                  {latestRun.stats.passed} Passed
                </span>
              </div>
              <div
                class="flex items-center gap-1.5 rounded-xl border border-rose-200 bg-rose-50 px-3 py-1.5 dark:border-rose-800/50 dark:bg-rose-900/20"
              >
                <XCircle size={13} class="text-rose-600 dark:text-rose-400" />
                <span
                  class="text-xs font-black text-rose-700 dark:text-rose-300"
                >
                  {latestRun.stats.failed} Failed
                </span>
              </div>
              <div
                class="flex items-center gap-1.5 rounded-xl border border-orange-200 bg-orange-50 px-3 py-1.5 dark:border-orange-800/50 dark:bg-orange-900/20"
              >
                <MinusCircle
                  size={13}
                  class="text-orange-600 dark:text-orange-400"
                />
                <span
                  class="text-xs font-black text-orange-700 dark:text-orange-300"
                >
                  {latestRun.stats.blocked} Blocked
                </span>
              </div>
              <div
                class="flex items-center gap-1.5 rounded-xl border border-indigo-200 bg-indigo-50 px-3 py-1.5 dark:border-indigo-800/50 dark:bg-indigo-900/20"
              >
                <Clock size={13} class="text-indigo-500 dark:text-indigo-400" />
                <span
                  class="text-xs font-black text-indigo-600 dark:text-indigo-300"
                >
                  {latestRun.stats.pending} Pending
                </span>
              </div>
              {#if latestRun.stats.skipped > 0}
                <div
                  class="flex items-center gap-1.5 rounded-xl border border-slate-200 bg-slate-50 px-3 py-1.5 dark:border-gray-700 dark:bg-gray-800"
                >
                  <SkipForward
                    size={13}
                    class="text-slate-500 dark:text-gray-400"
                  />
                  <span
                    class="text-xs font-black text-slate-600 dark:text-gray-300"
                  >
                    {latestRun.stats.skipped} Skipped
                  </span>
                </div>
              {/if}
              {#if latestRun.stats.invalid > 0}
                <div
                  class="flex items-center gap-1.5 rounded-xl border border-red-200 bg-red-50 px-3 py-1.5 dark:border-red-800/50 dark:bg-red-900/20"
                >
                  <AlertOctagon
                    size={13}
                    class="text-red-600 dark:text-red-400"
                  />
                  <span
                    class="text-xs font-black text-red-700 dark:text-red-300"
                  >
                    {latestRun.stats.invalid} Invalid
                  </span>
                </div>
              {/if}
              <div
                class="flex items-center gap-1.5 rounded-xl border border-slate-200 bg-white px-3 py-1.5 dark:border-gray-700 dark:bg-gray-800/60"
              >
                <Layers size={13} class="text-slate-400 dark:text-gray-400" />
                <span
                  class="text-xs font-black text-slate-500 dark:text-gray-400"
                >
                  {latestRun.stats.total} Total
                </span>
              </div>
            </div>

            {#if latestRun.description}
              <p
                class="text-xs font-medium text-slate-500 dark:text-gray-400 border-t border-slate-100 dark:border-gray-800 pt-3"
              >
                {latestRun.description}
              </p>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Search + Filters -->
    <div class="mb-8 flex flex-wrap items-center gap-3">
      <div
        class="flex h-10 flex-1 min-w-[280px] items-center gap-2 rounded-xl border border-slate-200 bg-white px-4 shadow-sm transition-all focus-within:border-indigo-500 focus-within:ring-4 focus-within:ring-indigo-500/10 dark:border-gray-700 dark:bg-gray-900"
      >
        <Search size={16} class="text-slate-400" />
        <input
          type="text"
          bind:value={query}
          on:input={handleSearch}
          placeholder="Search test cases..."
          class="flex-1 border-0 bg-transparent text-sm font-medium text-slate-700 outline-none placeholder:text-slate-400 dark:text-gray-200 dark:!bg-transparent"
        />
      </div>

      <select
        bind:value={filterPriority}
        on:change={() => loadData()}
        class="h-10 rounded-xl border border-slate-200 bg-white px-3 text-sm font-bold text-slate-600 shadow-sm dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300"
      >
        <option value="">All Priority</option>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
      </select>

      <select
        bind:value={filterStatus}
        on:change={() => loadData()}
        class="h-10 rounded-xl border border-slate-200 bg-white px-3 text-sm font-bold text-slate-600 shadow-sm dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300"
      >
        <option value="">All Status</option>
        <option value="actual">Actual</option>
        <option value="passed">Passed</option>
        <option value="failed">Failed</option>
        <option value="blocked">Blocked</option>
        <option value="draft">Draft</option>
        <option value="deprecated">Deprecated</option>
      </select>
    </div>

    <!-- Loading State -->
    {#if loading}
      <div class="flex items-center justify-center py-20">
        <div class="flex flex-col items-center gap-3">
          <div
            class="h-8 w-8 animate-spin rounded-full border-[3px] border-slate-200 border-t-indigo-600 dark:border-gray-700 dark:border-t-indigo-400"
          ></div>
          <p class="text-sm font-medium text-slate-500 dark:text-gray-400">
            Loading test cases...
          </p>
        </div>
      </div>
    {:else if suites.length === 0}
      <div class="flex items-center justify-center py-20">
        <div class="text-center">
          <p class="text-lg font-bold text-slate-400 dark:text-gray-500">
            No test cases found
          </p>
          <p class="mt-1 text-sm text-slate-400 dark:text-gray-600">
            There are no test suites or cases in this workspace.
          </p>
        </div>
      </div>
    {:else}
      <!-- Suite Tables -->
      <div class="space-y-6">
        {#each suites as suite}
          <section
            class="overflow-hidden rounded-2xl border border-slate-200/80 bg-white shadow-sm dark:border-gray-800 dark:bg-gray-900"
          >
            <!-- Suite Header -->
            <button
              class="flex w-full items-center gap-3 px-6 py-4 text-left transition-colors hover:bg-slate-50/50 dark:hover:bg-gray-800/50"
              on:click={() => toggleSuite(suite.id)}
            >
              {#if suite.collapsed}
                <ChevronRight size={16} class="text-slate-400 shrink-0" />
              {:else}
                <ChevronDown size={16} class="text-slate-400 shrink-0" />
              {/if}
              <h2 class="text-base font-black text-slate-800 dark:text-white">
                {suite.title}
              </h2>
              <span
                class="rounded-full bg-indigo-100 px-2.5 py-0.5 text-xs font-black text-indigo-600 dark:bg-indigo-900/30 dark:text-indigo-400"
              >
                {suite.case_count}
              </span>
            </button>

            <!-- Table -->
            {#if !suite.collapsed}
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead>
                    <tr
                      class="border-t border-slate-100 bg-slate-50/80 dark:border-gray-800 dark:bg-gray-800/50"
                    >
                      <th
                        class="px-6 py-3 text-left text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >ID</th
                      >
                      <th
                        class="px-6 py-3 text-left text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Title</th
                      >
                      <th
                        class="px-6 py-3 text-center text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Priority</th
                      >
                      <th
                        class="px-6 py-3 text-center text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Status</th
                      >
                      <th
                        class="px-6 py-3 text-center text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Fixed</th
                      >
                      {#if latestRun}
                        <th
                          class="px-6 py-3 text-center text-[10px] font-black uppercase tracking-wider text-indigo-400 dark:text-indigo-500"
                        >
                          Run Result
                        </th>
                      {/if}
                      <th
                        class="px-6 py-3 text-left text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Developer</th
                      >
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-100 dark:divide-gray-800">
                    {#each suite.cases as tc}
                      {@const runStatus = runCaseMap.get(tc.id)}
                      <tr
                        class="transition-colors hover:bg-slate-50/50 dark:hover:bg-gray-800/30 {runStatus ===
                        'failed'
                          ? 'bg-rose-50/30 dark:bg-rose-950/10'
                          : runStatus === 'passed'
                            ? 'bg-emerald-50/20 dark:bg-emerald-950/5'
                            : ''}"
                      >
                        <td class="whitespace-nowrap px-6 py-3">
                          <span
                            class="font-mono text-xs font-bold text-slate-400 dark:text-gray-500"
                            >TC-{tc.test_no}</span
                          >
                        </td>
                        <td class="px-6 py-3">
                          <span
                            class="text-sm font-semibold text-slate-700 dark:text-gray-200"
                            >{tc.title}</span
                          >
                        </td>
                        <td class="px-6 py-3 text-center">
                          <span
                            class="inline-block rounded-md border px-2 py-0.5 text-[10px] font-black uppercase {priorityBadge(
                              tc.priority,
                            )}"
                          >
                            {tc.priority}
                          </span>
                        </td>
                        <td class="px-6 py-3 text-center">
                          <span
                            class="inline-block rounded-md px-2.5 py-0.5 text-[10px] font-black uppercase {statusBadge(
                              tc.status,
                            )}"
                          >
                            {tc.status}
                          </span>
                        </td>
                        <td class="px-6 py-3 text-center">
                          <span
                            class="inline-block rounded-md px-2 py-0.5 text-[10px] font-black uppercase {fixedBadge(
                              tc.fixed,
                            )}"
                          >
                            {tc.fixed}
                          </span>
                        </td>
                        {#if latestRun}
                          <td class="px-6 py-3 text-center">
                            {#if runStatus}
                              <span
                                class="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[10px] font-black uppercase {runResultBadge(
                                  runStatus,
                                )}"
                              >
                                {#if runStatus === "passed"}
                                  <CheckCircle2 size={10} />
                                {:else if runStatus === "failed"}
                                  <XCircle size={10} />
                                {:else if runStatus === "blocked"}
                                  <MinusCircle size={10} />
                                {:else if runStatus === "skipped"}
                                  <SkipForward size={10} />
                                {:else if runStatus === "invalid"}
                                  <AlertOctagon size={10} />
                                {:else}
                                  <Clock size={10} />
                                {/if}
                                {runStatus}
                              </span>
                            {:else}
                              <span class="text-slate-300 dark:text-gray-700"
                                >—</span
                              >
                            {/if}
                          </td>
                        {/if}
                        <td class="px-6 py-3">
                          <span
                            class="text-xs font-medium text-slate-500 dark:text-gray-400"
                          >
                            {tc.assign_dev === "unassigned"
                              ? "—"
                              : tc.assign_dev}
                          </span>
                        </td>
                      </tr>
                    {/each}
                    {#if suite.cases.length === 0}
                      <tr>
                        <td
                          colspan={latestRun ? 7 : 6}
                          class="px-6 py-8 text-center text-sm font-medium text-slate-400 dark:text-gray-500"
                        >
                          No test cases in this suite
                        </td>
                      </tr>
                    {/if}
                  </tbody>
                </table>
              </div>
            {/if}
          </section>
        {/each}
      </div>
    {/if}
  </main>

  <!-- Footer -->
  <footer
    class="mt-16 border-t border-slate-200/80 bg-white/50 dark:border-gray-800/80 dark:bg-gray-950/50"
  >
    <div
      class="mx-auto flex max-w-7xl items-center justify-between px-6 py-6 text-xs font-medium text-slate-400 dark:text-gray-500"
    >
      <span>Public Test Case View — Read Only</span>
      <span>Powered by Khun Phaen</span>
    </div>
  </footer>
</div>
