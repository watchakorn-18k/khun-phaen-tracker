<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { api } from "$lib/apis";
  import { theme } from "$lib/stores/theme";
  import {
    Search,
    ChevronDown,
    ChevronRight,
    Filter,
    Moon,
    Sun,
    ExternalLink,
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

  $: workspaceId = $page.params.workspace_id;

  let suites: Suite[] = [];
  let loading = true;
  let query = "";
  let searchTimeout: any;
  let filterPriority = "";
  let filterStatus = "";

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
    };
  }

  async function loadData() {
    if (!workspaceId) return;
    loading = true;
    try {
      const suiteResp = await api.public.listSuites(workspaceId);
      if (!suiteResp.ok) return;
      const suiteData = await suiteResp.json();

      const suiteList = suiteData.map((s: any) => ({
        id: s.id || s._id,
        title: s.title,
        case_count: s.case_count ?? 0,
        collapsed: false,
        page: 1,
        hasMore: false,
        cases: [] as TestCase[],
      }));

      // Load cases per suite in parallel
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

  function handleSearch() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => loadData(), 300);
  }

  $: totalCases = suites.reduce((t, s) => t + (s.case_count || 0), 0);
  $: totalSuites = suites.filter((s) => s.id !== "unassigned").length;

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
      case "pass":
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
          class="flex-1 border-0 bg-transparent text-sm font-medium text-slate-700 outline-none placeholder:text-slate-400 dark:text-gray-200"
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
        <option value="pass">Pass</option>
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
                      <th
                        class="px-6 py-3 text-left text-[10px] font-black uppercase tracking-wider text-slate-400 dark:text-gray-500"
                        >Developer</th
                      >
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-100 dark:divide-gray-800">
                    {#each suite.cases as tc}
                      <tr
                        class="transition-colors hover:bg-slate-50/50 dark:hover:bg-gray-800/30"
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
                          colspan="6"
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
