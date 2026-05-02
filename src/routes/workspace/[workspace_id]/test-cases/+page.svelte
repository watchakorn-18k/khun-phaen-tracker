<script lang="ts">
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { currentWorkspaceName } from "$lib/stores/workspace";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { api } from "$lib/apis";
  import { createUIActions } from "$lib/stores/uiActions";
  import {
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronsUpDown,
    ClipboardList,
    Edit3,
    Folder,
    Image,
    ListChecks,
    MoreHorizontal,
    Plus,
    Search,
    Trash2,
    X,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  type Step = {
    action: string;
    data: string;
    expected: string;
  };

  type TestCase = {
    id: string;
    test_no: number;
    title: string;
    type: "manual" | "automated";
    priority: "high" | "medium" | "low";
    status: string;
    assignee?: string;
    description: string;
    preconditions?: string;
    postconditions?: string;
    steps: Step[];
    suite_id: string;
  };

  type Suite = {
    id: string;
    title: string;
    description: string;
    cases: TestCase[];
    page: number;
    hasMore: boolean;
  };

  type TestCaseForm = {
    name: string;
    suiteId: string;
    description: string;
    preconditions: string;
    postconditions: string;
    status: NonNullable<TestCase["status"]>;
    assignee: string;
  };

  type GherkinStep = {
    keyword: "given" | "when" | "then" | "and";
    text: string;
  };

  type ClassicStep = {
    action: string;
    data: string;
    expected: string;
  };

  let suites: Suite[] = [];
  const ui = createUIActions();

  let query = "";
  let field = "all fields";
  let selectedSuiteId = "";
  let selectedCaseId = "";
  let workspaceId = $page.params.workspace_id;
  let showDetail = true;
  let detailPanelWidth = 420;
  let stopPanelResize: (() => void) | null = null;
  let showTestCaseModal = false;
  let testCaseForm: TestCaseForm = {
    name: "",
    suiteId: "",
    description: "",
    preconditions: "",
    postconditions: "",
    status: "actual",
    assignee: "unassigned",
  };
  let testCaseSteps: GherkinStep[] = [
    { keyword: "given", text: "" },
    { keyword: "when", text: "" },
    { keyword: "then", text: "" },
  ];
  let stepFormat: "gherkin" | "classic" = "gherkin";
  let classicSteps: ClassicStep[] = [{ action: "", data: "", expected: "" }];
  let showCreateSuiteModal = false;
  let newSuiteName = "";
  let editingSuite: Suite | null = null;
  let showDeleteSuiteModal = false;
  let suiteToDelete: Suite | null = null;
  let deleteMode: "move" | "delete" = "move";

  let quickTestInputSuiteId = "";
  let quickTestTitle = "";

  async function handleCreateSuite() {
    if (!newSuiteName.trim() || !workspaceId) return;

    try {
      const resp = await api.data.testSuites.create(workspaceId, {
        title: newSuiteName,
      });
      if (resp.ok) {
        const newSuite = await resp.json();
        suites = [
          ...suites,
          {
            ...newSuite,
            id: newSuite.id || newSuite._id,
            cases: [],
            page: 1,
            hasMore: false,
          },
        ];
        newSuiteName = "";
        showCreateSuiteModal = false;
        ui.showMessage("Suite created successfully", "success");
      } else {
        ui.showMessage("Failed to create suite", "error");
      }
    } catch (e) {
      console.error("Create suite error:", e);
      ui.showMessage("An unexpected error occurred", "error");
    }
  }

  async function handleUpdateSuite() {
    if (!editingSuite || !editingSuite.title.trim()) return;
    try {
      const resp = await api.data.testSuites.update(editingSuite.id, {
        title: editingSuite.title,
      });
      if (resp.ok) {
        suites = suites.map((s) =>
          s.id === editingSuite?.id ? { ...s, title: editingSuite.title } : s,
        );
        ui.showMessage("Suite updated", "success");
        editingSuite = null;
      } else {
        ui.showMessage("Failed to update suite", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    }
  }

  async function handleDeleteSuite() {
    if (!suiteToDelete) return;
    try {
      const resp = await api.data.testSuites.delete(
        suiteToDelete.id,
        deleteMode,
      );
      if (resp.ok) {
        if (deleteMode === "move") {
          // Move cases to unassigned
          const unassignedSuite = suites.find((s) => s.id === "unassigned");
          const movedCases = suiteToDelete.cases || [];
          if (unassignedSuite) {
            suites = suites.map((s) =>
              s.id === "unassigned"
                ? { ...s, cases: [...s.cases, ...movedCases] }
                : s,
            );
          } else {
            suites = [
              ...suites,
              {
                id: "unassigned",
                title: "Unassigned",
                description: "",
                cases: movedCases,
                page: 1,
                hasMore: false,
              },
            ];
          }
        }
        suites = suites.filter((s) => s.id !== suiteToDelete?.id);
        ui.showMessage("Suite deleted", "success");
        showDeleteSuiteModal = false;
        suiteToDelete = null;
      } else {
        ui.showMessage("Failed to delete suite", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    }
  }

  function mapBackendToFrontend(tc: any): TestCase {
    return {
      id: tc.id || tc._id,
      test_no: tc.test_no,
      title: tc.name,
      type: "manual", // Default for now
      priority: tc.priority || "medium",
      status: tc.status,
      assignee: tc.assign_tester || "unassigned",
      description: tc.description || "",
      preconditions: tc.preconditions || "",
      postconditions: tc.postconditions || "",
      steps:
        tc.classic_steps ||
        (tc.gherkin_steps || []).map((s: any) => ({
          action: `${s.keyword} ${s.text}`,
          data: "",
          expected: "",
        })),
      suite_id: tc.suite_id,
    };
  }

  onMount(async () => {
    if (workspaceId) {
      try {
        const [suiteResp, caseResp] = await Promise.all([
          api.data.testSuites.list(workspaceId),
          api.data.testCases.list(workspaceId),
        ]);

        if (suiteResp.ok && caseResp.ok) {
          const suiteData = await suiteResp.json();
          const caseData = await caseResp.json();

          const mappedCases = caseData.map(mapBackendToFrontend);

          // Map suites and ensure they have 'id'
          const suitesWithId = suiteData.map((s: any) => ({
            ...s,
            id: s.id || s._id,
          }));

          const suiteIds = new Set(suitesWithId.map((s: any) => s.id));
          const unassignedCases = mappedCases.filter(
            (c: TestCase) => !suiteIds.has(c.suite_id),
          );

          suites = suitesWithId.map((s: any) => ({
            ...s,
            cases: mappedCases.filter((c: TestCase) => c.suite_id === s.id),
            page: 1,
            hasMore:
              mappedCases.filter((c: TestCase) => c.suite_id === s.id).length >=
              10,
          }));

          // Add 'Unassigned' virtual suite if there are unassigned cases
          if (unassignedCases.length > 0) {
            suites = [
              ...suites,
              {
                id: "unassigned",
                title: "Unassigned",
                description: "",
                cases: unassignedCases,
                page: 1,
                hasMore: unassignedCases.length >= 10,
              },
            ];
          }

          if (suites.length > 0) {
            selectedSuiteId = suites[0].id;
            testCaseForm.suiteId =
              suites[0].id === "unassigned" ? "" : suites[0].id;
            if (suites[0].cases.length > 0) {
              selectedCaseId = suites[0].cases[0].id;
            }
          }
        }
      } catch (e) {
        console.error("Failed to load test cases:", e);
      }
    }
  });
  async function loadSuitePage(suiteId: string, page: number) {
    if (!workspaceId) return;
    try {
      const resp = await api.data.testCases.list(workspaceId, {
        suite_id: suiteId === "unassigned" ? "none" : suiteId,
        page: page,
        limit: 10,
      });
      if (resp.ok) {
        const data = await resp.json();
        const mapped = data.map(mapBackendToFrontend);
        suites = suites.map((s) => {
          if (s.id === suiteId) {
            return {
              ...s,
              cases: mapped,
              page: page,
              hasMore: mapped.length >= 10,
            };
          }
          return s;
        });
      }
    } catch (e) {
      console.error("Failed to load suite page:", e);
    }
  }
  const assignees = [
    { id: "unassigned", name: "Unassigned" },
    { id: "wk-18k", name: "WK 18K" },
    { id: "qa-lead", name: "QA Lead" },
    { id: "product-owner", name: "Product Owner" },
  ];

  const fieldOptions = [
    { value: "all fields", label: "By all fields" },
    { value: "title", label: "By title" },
    { value: "steps", label: "By steps" },
  ];

  const statusOptions = [
    { value: "draft", label: "Draft" },
    { value: "actual", label: "Actual" },
    { value: "failed", label: "Failed" },
    { value: "blocked", label: "Blocked" },
    { value: "deprecated", label: "Deprecated" },
  ];

  const gherkinKeywordOptions = [
    { value: "given", label: "Given" },
    { value: "when", label: "When" },
    { value: "then", label: "Then" },
    { value: "and", label: "And" },
  ];

  const stepFormatOptions = [
    { value: "gherkin", label: "Gherkin" },
    { value: "classic", label: "Classic" },
  ];

  $: suiteOptions = suites.map((suite) => ({
    value: suite.id,
    label: suite.title,
  }));
  $: assigneeOptions = assignees.map((assignee) => ({
    value: assignee.id,
    label: assignee.name,
  }));

  $: workspaceId = $page.params.workspace_id;
  $: workspaceLabel = $currentWorkspaceName || "Current workspace";
  $: suiteCount = suites.length;
  $: caseCount = suites.reduce(
    (total, suite) => total + (suite.cases?.length || 0),
    0,
  );
  $: selectedSuite =
    suites.find((suite) => suite.id === selectedSuiteId) || suites[0];
  $: selectedCase =
    suites
      .flatMap((suite) => suite.cases || [])
      .find((testCase) => testCase.id === selectedCaseId) ||
    (selectedSuite?.cases ? selectedSuite.cases[0] : null);
  $: filteredSuites = suites
    .map((suite) => ({
      ...suite,
      cases: (suite.cases || []).filter((testCase) =>
        `${suite.title} ${testCase.id} ${testCase.title} ${testCase.description}`
          .toLowerCase()
          .includes(query.trim().toLowerCase()),
      ),
    }))
    .filter(
      (suite) =>
        (suite.cases?.length || 0) > 0 ||
        suite.title.toLowerCase().includes(query.trim().toLowerCase()),
    );

  function selectSuite(suite: Suite) {
    selectedSuiteId = suite.id;
    selectedCaseId = suite.cases[0]?.id || selectedCaseId;
    showDetail = true;
  }

  function selectCase(suite: Suite, testCase: TestCase) {
    selectedSuiteId = suite.id;
    selectedCaseId = testCase.id;
    showDetail = true;
  }

  function getEditorUrl(suiteId = selectedSuiteId, caseId = "") {
    const params = new URLSearchParams($page.url.searchParams);
    params.set("suite", suiteId);
    if (caseId) params.set("case", caseId);
    else params.delete("case");
    return `${base}/workspace/${workspaceId}/test-cases/editor?${params.toString()}`;
  }

  function openTestCaseEditor(suiteId = selectedSuiteId, caseId = "") {
    goto(getEditorUrl(suiteId, caseId));
  }

  function openTestCaseModal(suiteId = selectedSuiteId) {
    testCaseForm = {
      name: "",
      suiteId,
      description: "",
      preconditions: "",
      postconditions: "",
      status: "actual",
      assignee: "unassigned",
    };
    testCaseSteps = [
      { keyword: "given", text: "" },
      { keyword: "when", text: "" },
      { keyword: "then", text: "" },
    ];
    stepFormat = "gherkin";
    classicSteps = [{ action: "", data: "", expected: "" }];
    showTestCaseModal = true;
  }

  function closeTestCaseModal() {
    showTestCaseModal = false;
  }

  function addTestCaseStep() {
    const nextKeyword =
      testCaseSteps.length === 0
        ? "given"
        : testCaseSteps.length % 3 === 1
          ? "when"
          : testCaseSteps.length % 3 === 2
            ? "then"
            : "given";
    testCaseSteps = [...testCaseSteps, { keyword: nextKeyword, text: "" }];
  }

  function removeTestCaseStep(index: number) {
    testCaseSteps = testCaseSteps.filter((_, stepIndex) => stepIndex !== index);
  }

  function addClassicStep() {
    classicSteps = [...classicSteps, { action: "", data: "", expected: "" }];
  }

  function removeClassicStep(index: number) {
    classicSteps = classicSteps.filter((_, stepIndex) => stepIndex !== index);
  }

  function getSubmittedSteps(): Step[] {
    if (stepFormat === "classic") {
      return classicSteps
        .filter(
          (step) =>
            step.action.trim() || step.data.trim() || step.expected.trim(),
        )
        .map((step) => ({
          action: step.action.trim(),
          data: step.data.trim(),
          expected: step.expected.trim(),
        }));
    }

    return testCaseSteps
      .filter((step) => step.text.trim())
      .map((step) => ({
        action: `${gherkinKeywordOptions.find((option) => option.value === step.keyword)?.label || "Step"} ${step.text.trim()}`,
        data: "",
        expected: "",
      }));
  }

  async function submitTestCase() {
    if (!testCaseForm.name.trim() || !workspaceId) return;

    const payload = {
      suite_id: testCaseForm.suiteId || suites[0]?.id || null,
      name: testCaseForm.name.trim(),
      description: testCaseForm.description.trim(),
      preconditions: testCaseForm.preconditions.trim(),
      postconditions: testCaseForm.postconditions.trim(),
      status: testCaseForm.status,
      assign_tester: testCaseForm.assignee,
      step_format: stepFormat,
      classic_steps: stepFormat === "classic" ? classicSteps : undefined,
      gherkin_steps:
        stepFormat === "gherkin"
          ? testCaseSteps.map((s) => ({ keyword: s.keyword, text: s.text }))
          : undefined,
    };

    try {
      const resp = await api.data.testCases.create(workspaceId, payload);
      if (resp.ok) {
        const result = await resp.json();
        const newCase = mapBackendToFrontend(result);

        suites = suites.map((suite) =>
          suite.id === newCase.suite_id
            ? { ...suite, cases: [...(suite.cases || []), newCase] }
            : suite,
        );

        selectedSuiteId = newCase.suite_id;
        selectedCaseId = newCase.id;
        showDetail = true;
        ui.showMessage("Test case created", "success");
        closeTestCaseModal();
      } else {
        ui.showMessage("Failed to create test case", "error");
      }
    } catch (e) {
      console.error("Submit error:", e);
      ui.showMessage("An error occurred", "error");
    }
  }

  async function handleQuickCreate(suiteId: string) {
    if (!quickTestTitle.trim() || !workspaceId) {
      quickTestInputSuiteId = "";
      quickTestTitle = "";
      return;
    }

    const payload = {
      suite_id: suiteId === "unassigned" ? null : suiteId,
      name: quickTestTitle.trim(),
      description: "",
      preconditions: "",
      postconditions: "",
      status: "actual",
      assign_tester: "unassigned",
      step_format: "gherkin",
      gherkin_steps: [],
    };

    try {
      const resp = await api.data.testCases.create(workspaceId, payload);
      if (resp.ok) {
        const result = await resp.json();
        let newTcData = result;

        // Ensure we have a test number, try to fetch if not in result
        if (!newTcData.test_no) {
          const nextResp = await api.data.testCases.nextNumber(workspaceId);
          if (nextResp.ok) {
            const nextData = await nextResp.json();
            newTcData.test_no = (nextData.next_number || 1) - 1;
          }
        }

        const newCase = mapBackendToFrontend(newTcData);

        suites = suites.map((suite) =>
          suite.id === (newCase.suite_id || "unassigned")
            ? { ...suite, cases: [...(suite.cases || []), newCase] }
            : suite,
        );

        selectedSuiteId = newCase.suite_id || "unassigned";
        selectedCaseId = newCase.id;
        showDetail = true;
        ui.showMessage($_("testCases__created_success"), "success");
      } else {
        ui.showMessage($_("testCases__created_error"), "error");
      }
    } catch (e) {
      console.error("Quick create error:", e);
      ui.showMessage($_("page__error"), "error");
    } finally {
      quickTestInputSuiteId = "";
      quickTestTitle = "";
    }
  }

  function priorityColor(priority: TestCase["priority"]) {
    if (priority === "high")
      return "text-rose-500 bg-rose-50 border-rose-100 dark:bg-rose-500/10 dark:border-rose-500/20";
    if (priority === "medium")
      return "text-amber-600 bg-amber-50 border-amber-100 dark:bg-amber-500/10 dark:border-amber-500/20";
    return "text-gray-500 bg-gray-50 border-gray-200 dark:bg-gray-800 dark:border-gray-700";
  }

  function clampDetailWidth(width: number) {
    if (!browser) return width;
    const maxWidth = Math.max(420, window.innerWidth - 520);
    return Math.min(Math.max(width, 360), maxWidth);
  }

  function startPanelResize(event: PointerEvent) {
    if (!browser) return;
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = detailPanelWidth;

    const handlePointerMove = (moveEvent: PointerEvent) => {
      detailPanelWidth = clampDetailWidth(
        startWidth + startX - moveEvent.clientX,
      );
    };

    const handlePointerUp = () => {
      localStorage.setItem(
        "test-case-detail-panel-width",
        String(detailPanelWidth),
      );
      stopPanelResize?.();
      stopPanelResize = null;
    };

    stopPanelResize?.();
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp, { once: true });
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    stopPanelResize = () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  }

  onMount(() => {
    if (!browser) return;
    const savedWidth = Number(
      localStorage.getItem("test-case-detail-panel-width"),
    );
    if (Number.isFinite(savedWidth) && savedWidth > 0) {
      detailPanelWidth = clampDetailWidth(savedWidth);
    }
  });

  onDestroy(() => {
    stopPanelResize?.();
  });
  function autofocus(node: HTMLElement) {
    node.focus();
  }
</script>

<svelte:head>
  <title>Test Cases - {workspaceLabel}</title>
</svelte:head>

<div
  class="h-full min-h-screen bg-white text-slate-900 dark:bg-gray-950 dark:text-gray-100"
>
  <div class="flex h-screen min-w-0 overflow-hidden">
    <section
      class="flex min-w-0 flex-1 flex-col border-r border-gray-200 dark:border-gray-800"
    >
      <header
        class="shrink-0 border-b border-gray-200 bg-white px-6 py-5 dark:border-gray-800 dark:bg-gray-950"
      >
        <div
          class="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between"
        >
          <div>
            <p
              class="text-[11px] font-black uppercase tracking-[0.24em] text-slate-400"
            >
              Workspace test repository
            </p>
            <h1
              class="mt-1 text-2xl font-black tracking-tight text-slate-800 dark:text-white"
            >
              {workspaceLabel} repository
            </h1>
            <p class="mt-1 text-sm font-medium text-slate-500">
              {caseCount} cases ({caseCount}) | {suiteCount} suites ({suiteCount})
              · {workspaceId}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <div
              class="flex h-9 min-w-[280px] overflow-hidden rounded-lg border border-slate-300 bg-white dark:border-gray-700 dark:bg-gray-900"
            >
              <label class="flex min-w-0 flex-1 items-center gap-2 px-3">
                <Search size={16} class="shrink-0 text-slate-400" />
                <input
                  bind:value={query}
                  class="min-w-0 flex-1 border-0 bg-transparent text-sm font-medium outline-none placeholder:text-slate-400"
                  placeholder="Search"
                />
              </label>
              <div class="w-44 border-l border-slate-300 dark:border-gray-700">
                <SearchableSelect
                  id="test-case-search-field"
                  bind:value={field}
                  options={fieldOptions}
                  showSearch={false}
                  minimal={true}
                />
              </div>
            </div>
            <button
              class="h-9 rounded-lg px-3 text-sm font-black text-indigo-600 hover:bg-indigo-50 dark:text-indigo-400 dark:hover:bg-indigo-500/10"
            >
              Add filter
            </button>
          </div>
        </div>
      </header>

      <div class="flex min-h-0 flex-1">
        <aside
          class="hidden w-64 shrink-0 border-r border-gray-200 bg-white p-4 lg:block dark:border-gray-800 dark:bg-gray-950"
        >
          <div class="mb-3 flex items-center gap-2">
            <button
              class="grid h-8 w-8 place-items-center rounded-lg bg-slate-100 text-slate-500 dark:bg-gray-800 dark:text-gray-300"
              title="Toggle suites"
            >
              <ChevronsUpDown size={16} />
            </button>
            <h2 class="text-lg font-black text-slate-800 dark:text-white">
              Suites
            </h2>
            <button
              class="grid h-8 w-8 place-items-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
              title="Add suite"
              on:click={() => (showCreateSuiteModal = true)}
            >
              <Plus size={16} />
            </button>
          </div>

          <div class="space-y-1">
            {#each suites as suite}
              <button
                class="group flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm transition-colors {selectedSuiteId ===
                suite.id
                  ? 'bg-slate-100 font-black text-slate-800 dark:bg-gray-800 dark:text-white'
                  : 'font-bold text-slate-600 hover:bg-slate-50 dark:text-gray-400 dark:hover:bg-gray-900'}"
                on:click={() => selectSuite(suite)}
              >
                <ChevronDown size={14} class="shrink-0 text-slate-500" />
                <span class="min-w-0 flex-1 truncate">{suite.title}</span>
                <span class="text-xs font-black text-slate-500"
                  >{suite.cases?.length || 0}</span
                >
                <MoreHorizontal
                  size={14}
                  class="text-slate-400 opacity-0 transition-opacity group-hover:opacity-100"
                />
              </button>
            {/each}
          </div>
        </aside>

        <main
          class="min-w-0 flex-1 overflow-y-auto bg-white p-4 dark:bg-gray-950"
        >
          <div class="space-y-7">
            {#each filteredSuites as suite}
              <section>
                <div
                  class="flex min-h-12 items-center gap-3 rounded-t-lg bg-slate-100 px-5 dark:bg-gray-900"
                >
                  <h3
                    class="min-w-0 flex-1 truncate text-base font-black text-slate-700 dark:text-white"
                  >
                    {suite.title}
                  </h3>
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md text-indigo-600 hover:bg-white dark:text-indigo-400 dark:hover:bg-gray-800"
                    title="Add case"
                    on:click={() => openTestCaseEditor(suite.id)}
                  >
                    <Plus size={16} />
                  </button>
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md text-slate-400 hover:bg-white hover:text-slate-700 dark:hover:bg-gray-800 dark:hover:text-white"
                    title="Edit suite"
                    on:click={() => (editingSuite = { ...suite })}
                  >
                    <Edit3 size={15} />
                  </button>
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md text-slate-400 hover:bg-white hover:text-rose-600 dark:hover:bg-gray-800 dark:hover:text-rose-400"
                    title="Delete suite"
                    on:click={() => {
                      suiteToDelete = suite;
                      showDeleteSuiteModal = true;
                    }}
                  >
                    <Trash2 size={15} />
                  </button>
                </div>

                <div class="divide-y divide-slate-200 dark:divide-gray-800">
                  {#each suite.cases || [] as testCase}
                    <button
                      class="grid w-full grid-cols-[24px_52px_minmax(0,1fr)_auto] items-center gap-3 rounded-md px-3 py-2 text-left transition-colors {selectedCaseId ===
                      testCase.id
                        ? 'bg-slate-100 dark:bg-transparent'
                        : 'hover:bg-slate-50 dark:hover:bg-gray-900/70'}"
                      on:click={() => selectCase(suite, testCase)}
                    >
                      <span class="grid h-5 w-5 place-items-center">
                        <span
                          class="h-2.5 w-2.5 rounded-full border-2 {testCase.priority ===
                          'high'
                            ? 'border-rose-500'
                            : 'border-slate-300'}"
                        ></span>
                      </span>
                      <span class="font-mono text-sm font-bold text-slate-400"
                        >TC-{testCase.test_no}</span
                      >
                      <span
                        class="min-w-0 truncate text-sm font-bold text-slate-700 dark:text-gray-200"
                        >{testCase.title}</span
                      >
                      <span
                        class="hidden rounded-md border px-2 py-0.5 text-[11px] font-black uppercase tracking-wide sm:inline-flex {priorityColor(
                          testCase.priority,
                        )}"
                      >
                        {testCase.priority}
                      </span>
                    </button>
                  {/each}

                  {#if suite.cases.length === 0}
                    <div
                      class="flex items-center justify-center py-4 opacity-50"
                    >
                      <p class="text-[12px] font-medium text-slate-400">
                        {$_("testCases__empty_suite")}
                      </p>
                    </div>
                  {/if}
                </div>

                <div
                  class="flex items-center justify-between px-3 py-3 border-t border-slate-100 dark:border-gray-800"
                >
                  <div class="flex flex-1 items-center gap-4">
                    {#if quickTestInputSuiteId === suite.id}
                      <input
                        bind:value={quickTestTitle}
                        class="flex-1 border-0 !bg-transparent text-sm font-bold outline-none placeholder:text-slate-400 dark:text-white"
                        placeholder={$_("testCases__title_placeholder")}
                        on:keydown={(e) => {
                          if (e.key === "Enter") handleQuickCreate(suite.id);
                          if (e.key === "Escape") {
                            quickTestInputSuiteId = "";
                            quickTestTitle = "";
                          }
                        }}
                        on:blur={() => {
                          if (!quickTestTitle.trim())
                            quickTestInputSuiteId = "";
                        }}
                        use:autofocus
                      />
                    {:else}
                      <button
                        class="text-sm font-black text-indigo-600 hover:text-indigo-700 dark:text-indigo-400 dark:hover:text-indigo-300"
                        on:click={() => {
                          quickTestInputSuiteId = suite.id;
                          quickTestTitle = "";
                        }}
                      >
                        {$_("testCases__quick_create")}
                      </button>
                    {/if}

                    <div class="flex gap-2">
                      {#if suite.page > 1 || suite.hasMore}
                        <button
                          class="flex items-center gap-1 rounded-md border border-slate-200 px-3 py-1.5 text-xs font-bold text-slate-600 transition-colors hover:bg-slate-50 disabled:opacity-30 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900"
                          disabled={suite.page <= 1}
                          on:click={() =>
                            loadSuitePage(suite.id, suite.page - 1)}
                        >
                          <ChevronLeft size={14} />
                          Previous
                        </button>
                        <button
                          class="flex items-center gap-1 rounded-md border border-slate-200 px-3 py-1.5 text-xs font-bold text-slate-600 transition-colors hover:bg-slate-50 disabled:opacity-30 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900"
                          disabled={!suite.hasMore}
                          on:click={() =>
                            loadSuitePage(suite.id, suite.page + 1)}
                        >
                          Next
                          <ChevronRight size={14} />
                        </button>
                      {/if}
                    </div>
                  </div>
                </div>
              </section>
            {/each}
          </div>
        </main>
      </div>
    </section>

    {#if showDetail && selectedCase}
      <aside
        class="relative hidden shrink-0 overflow-y-auto border-l border-gray-200 bg-white xl:block dark:border-gray-800 dark:bg-gray-950"
        style="width: {detailPanelWidth}px"
      >
        <button
          class="absolute left-0 top-0 z-20 h-full w-2 -translate-x-1 cursor-col-resize border-0 bg-transparent transition-colors hover:bg-indigo-500/10 focus:bg-indigo-500/10 focus:outline-none"
          aria-label="Resize test case details panel"
          title="Resize details panel"
          on:pointerdown={startPanelResize}
        ></button>
        <div
          class="sticky top-0 z-10 border-b border-gray-200 bg-white px-5 py-4 dark:border-gray-800 dark:bg-gray-950"
        >
          <div class="flex items-start gap-3">
            <div class="min-w-0 flex-1">
              <p class="font-mono text-sm font-bold text-slate-400">
                TC-{selectedCase.test_no}
              </p>
              <h2
                class="mt-1 truncate text-2xl font-black text-slate-800 dark:text-white"
              >
                {selectedCase.title}
              </h2>
              <p class="mt-1 text-sm font-semibold text-slate-500">
                {selectedSuite.title}
              </p>
            </div>
            <button
              class="grid h-9 w-9 place-items-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
              title="Close details"
              on:click={() => (showDetail = false)}
            >
              <X size={18} />
            </button>
          </div>

          <div class="mt-4 flex flex-wrap items-center gap-2">
            <button
              class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-200"
              title="Edit"
              on:click={() =>
                openTestCaseEditor(selectedSuite.id, selectedCase.id)}
            >
              <Edit3 size={17} />
            </button>
            <button
              class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-200"
              title="Delete"
            >
              <Trash2 size={17} />
            </button>
          </div>

          <div
            class="mt-4 flex gap-6 border-b border-gray-200 text-sm font-black text-slate-500 dark:border-gray-800"
          >
            <span
              class="border-b-2 border-indigo-600 pb-2 text-slate-800 dark:text-white"
              >General</span
            >
          </div>
        </div>

        <div class="space-y-7 px-5 py-6">
          <section>
            <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
              Description
            </h3>
            <p class="mt-2 text-sm leading-6 text-slate-700 dark:text-gray-300">
              {selectedCase.description}
            </p>
          </section>

          <section class="grid grid-cols-1 gap-5">
            <div>
              <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
                Pre-conditions
              </h3>
              <p class="mt-2 text-sm font-medium text-slate-500">
                {selectedCase.preconditions || "Not set"}
              </p>
            </div>
            <div>
              <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
                Post-conditions
              </h3>
              <p class="mt-2 text-sm font-medium text-slate-500">
                {selectedCase.postconditions || "Not set"}
              </p>
            </div>
          </section>

          <section>
            <div class="mb-4 flex items-center justify-between">
              <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
                Steps
              </h3>
              <button
                class="inline-flex items-center gap-1 rounded-lg px-2 py-1 text-xs font-black text-indigo-600 hover:bg-indigo-50 dark:text-indigo-400 dark:hover:bg-indigo-500/10"
              >
                <Plus size={14} />
                Add step
              </button>
            </div>

            <div class="space-y-5">
              {#each selectedCase.steps || [] as step, index}
                <div class="grid grid-cols-[28px_minmax(0,1fr)_76px] gap-3">
                  <div
                    class="pt-2 text-center text-sm font-bold text-slate-600 dark:text-gray-300"
                  >
                    {index + 1}
                  </div>
                  <div class="space-y-3">
                    <div
                      class="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-800 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-200"
                    >
                      {step.action}
                    </div>
                    <div
                      class="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-500 dark:border-gray-700 dark:bg-gray-900"
                    >
                      {step.data}
                    </div>
                    <div
                      class="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-500 dark:border-gray-700 dark:bg-gray-900"
                    >
                      {step.expected}
                    </div>
                  </div>
                  <div class="flex items-start gap-2 pt-1">
                    <button
                      class="grid h-8 w-8 place-items-center rounded-md bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-200"
                      title="Attach image"
                    >
                      <Image size={15} />
                    </button>
                    <button
                      class="grid h-8 w-8 place-items-center rounded-md bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-200"
                      title="More"
                    >
                      <MoreHorizontal size={15} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        </div>
      </aside>
    {/if}
  </div>
</div>

{#if showTestCaseModal}
  <div class="fixed inset-0 z-[120] flex items-center justify-center px-4 py-8">
    <button
      class="absolute inset-0 bg-slate-950/70 backdrop-blur-sm"
      aria-label="Close test case dialog"
      on:click={closeTestCaseModal}
    ></button>

    <form
      class="relative z-10 flex max-h-[90vh] min-h-[560px] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-white/10 bg-[#111827] text-white shadow-2xl shadow-black/40"
      on:submit|preventDefault={submitTestCase}
    >
      <div class="flex items-center justify-between px-5 py-4 shrink-0">
        <div
          class="flex min-w-0 items-center gap-2 text-[13px] font-medium text-gray-500"
        >
          <span class="truncate hover:text-gray-300 transition-colors"
            >{workspaceLabel}</span
          >
          <span class="text-gray-600">›</span>
          <span class="text-gray-400">Create test case</span>
        </div>
        <button
          type="button"
          class="p-1.5 text-gray-400 hover:text-white hover:!bg-transparent rounded-md transition-all"
          title="Close"
          on:click={closeTestCaseModal}
        >
          <X size={18} />
        </button>
      </div>

      <div class="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-5 py-2">
        <div class="space-y-6">
          <div>
            <input
              bind:value={testCaseForm.name}
              class="w-full !bg-transparent px-0 py-2 text-xl font-bold text-white placeholder:text-gray-600 border-none outline-none"
              placeholder="Example: Validate invite-link signup flow..."
            />
          </div>

          <div class="min-h-[120px]">
            <textarea
              bind:value={testCaseForm.description}
              class="min-h-[120px] w-full resize-none !bg-transparent px-0 text-[15px] leading-relaxed text-gray-300 placeholder:text-gray-600 border-none outline-none"
              placeholder="Additional details..."
            ></textarea>
          </div>

          <div class="flex flex-wrap items-center gap-2 pt-2 pb-4">
            <div class="property-select min-w-[150px]">
              <SearchableSelect
                id="test-case-suite"
                bind:value={testCaseForm.suiteId}
                options={suiteOptions}
                placeholder="Search suites..."
                emptyText="No suites found"
                minimal={true}
              />
            </div>

            <div class="property-select min-w-[170px]">
              <SearchableSelect
                id="test-case-status"
                bind:value={testCaseForm.status}
                options={statusOptions}
                showSearch={false}
                minimal={true}
              />
            </div>

            <div class="property-select min-w-[150px]">
              <SearchableSelect
                id="test-case-assignee"
                bind:value={testCaseForm.assignee}
                options={assigneeOptions}
                placeholder="Search assignees..."
                emptyText="No assignees found"
                minimal={true}
              />
            </div>
          </div>

          <div class="border-t border-white/5 pt-5">
            <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
              <label>
                <span
                  class="text-[12px] font-black uppercase tracking-widest text-gray-500"
                  >Pre-conditions</span
                >
                <textarea
                  bind:value={testCaseForm.preconditions}
                  class="mt-3 min-h-28 w-full resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                  placeholder="Conditions required before this test starts..."
                ></textarea>
              </label>

              <label>
                <span
                  class="text-[12px] font-black uppercase tracking-widest text-gray-500"
                  >Post-conditions</span
                >
                <textarea
                  bind:value={testCaseForm.postconditions}
                  class="mt-3 min-h-28 w-full resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                  placeholder="Expected state after this test finishes..."
                ></textarea>
              </label>
            </div>
          </div>

          <div class="border-t border-white/5 pt-5">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div class="flex items-center gap-2">
                <h3 class="text-sm font-black text-gray-200">
                  Test Case Steps
                </h3>
                <div class="property-select min-w-[96px]">
                  <SearchableSelect
                    id="test-case-step-format"
                    bind:value={stepFormat}
                    options={stepFormatOptions}
                    showSearch={false}
                    minimal={true}
                  />
                </div>
              </div>
              {#if stepFormat === "gherkin"}
                <div
                  class="flex items-center gap-4 text-xs font-black text-gray-500"
                >
                  <span>Raw</span>
                  <span class="text-indigo-400">Steps</span>
                </div>
              {/if}
            </div>

            {#if stepFormat === "classic"}
              <div class="space-y-4">
                {#each classicSteps as step, index}
                  <div
                    class="grid grid-cols-[28px_minmax(0,1fr)_minmax(0,1fr)_minmax(0,1fr)_72px] items-start gap-3"
                  >
                    <div
                      class="pt-2 text-center text-sm font-black text-gray-400"
                    >
                      {index + 1}
                    </div>
                    <textarea
                      bind:value={step.action}
                      class="min-h-9 w-full resize-y rounded-lg border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                      placeholder="Action"
                    ></textarea>
                    <textarea
                      bind:value={step.data}
                      class="min-h-9 w-full resize-y rounded-lg border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                      placeholder="Data"
                    ></textarea>
                    <textarea
                      bind:value={step.expected}
                      class="min-h-9 w-full resize-y rounded-lg border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                      placeholder="Expected result"
                    ></textarea>
                    <div class="flex items-center gap-2 pt-0.5">
                      <button
                        type="button"
                        class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:!bg-transparent hover:text-gray-200"
                        title="Attach image"
                      >
                        <Image size={15} />
                      </button>
                      <button
                        type="button"
                        class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:!bg-transparent hover:text-rose-400"
                        title="Delete step"
                        on:click={() => removeClassicStep(index)}
                      >
                        <Trash2 size={15} />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>

              <div class="mt-4 flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-gray-200 hover:!bg-transparent hover:border-white/20"
                  on:click={addClassicStep}
                >
                  <Plus size={15} />
                  New step
                </button>
                <button
                  type="button"
                  class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-gray-200 hover:!bg-transparent hover:border-white/20"
                >
                  <Plus size={15} />
                  New shared step
                </button>
              </div>
            {:else}
              <div class="space-y-3">
                {#each testCaseSteps as step, index}
                  <div
                    class="grid grid-cols-[28px_104px_minmax(0,1fr)_32px] items-center gap-3"
                  >
                    <div class="text-center text-sm font-black text-gray-400">
                      {index + 1}
                    </div>
                    <div class="property-select">
                      <SearchableSelect
                        id={`test-case-step-keyword-${index}`}
                        bind:value={step.keyword}
                        options={gherkinKeywordOptions}
                        showSearch={false}
                        minimal={true}
                      />
                    </div>
                    <input
                      bind:value={step.text}
                      class="h-9 min-w-0 rounded-lg border border-white/10 !bg-transparent px-3 text-sm font-medium text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                      placeholder="Write step..."
                    />
                    <button
                      type="button"
                      class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:!bg-transparent hover:text-rose-400"
                      title="Delete step"
                      on:click={() => removeTestCaseStep(index)}
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                {/each}
              </div>

              <button
                type="button"
                class="mt-4 inline-flex items-center gap-1.5 rounded-lg px-1 py-1 text-sm font-black text-indigo-400 hover:text-indigo-300"
                on:click={addTestCaseStep}
              >
                <Plus size={15} />
                Add step
              </button>
            {/if}
          </div>
        </div>
      </div>

      <div
        class="flex items-center justify-end gap-2 border-t border-white/5 px-5 py-4"
      >
        <button
          type="button"
          class="h-10 rounded-lg px-4 text-sm font-black text-gray-400 hover:!bg-transparent hover:text-white"
          on:click={closeTestCaseModal}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="h-10 rounded-xl bg-indigo-600 px-6 text-sm font-black text-white shadow-lg shadow-indigo-600/25 hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-indigo-500/40"
          disabled={!testCaseForm.name.trim()}
        >
          Create test case
        </button>
      </div>
    </form>
  </div>
{/if}

{#if showCreateSuiteModal}
  <div
    class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
    transition:fade={{ duration: 150 }}
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="absolute inset-0 bg-black/60 backdrop-blur-sm"
      on:click={() => (showCreateSuiteModal = false)}
    ></div>
    <div
      class="relative w-full max-w-md rounded-2xl border border-gray-200 bg-white p-6 shadow-2xl dark:border-gray-800 dark:bg-gray-900"
    >
      <h3 class="text-lg font-black text-slate-800 dark:text-white">
        Create New Suite
      </h3>
      <p class="mt-1 text-sm text-slate-500">
        Enter a name for the new test suite.
      </p>

      <div class="mt-6">
        <label
          for="new-suite-name"
          class="block text-[12px] font-black uppercase tracking-widest text-slate-500"
        >
          Suite Name
        </label>
        <input
          id="new-suite-name"
          type="text"
          bind:value={newSuiteName}
          class="mt-3 w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-800 outline-none focus:border-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:focus:border-indigo-400"
          placeholder="e.g. Authentication, Dashboard..."
          autofocus
          on:keydown={(e) => e.key === "Enter" && handleCreateSuite()}
        />
      </div>

      <div class="mt-8 flex items-center justify-end gap-3">
        <button
          type="button"
          class="rounded-xl px-4 py-2 text-sm font-bold text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => (showCreateSuiteModal = false)}
        >
          Cancel
        </button>
        <button
          type="button"
          class="rounded-xl bg-indigo-600 px-6 py-2 text-sm font-black text-white hover:bg-indigo-500 disabled:opacity-50"
          disabled={!newSuiteName.trim()}
          on:click={handleCreateSuite}
        >
          Create Suite
        </button>
      </div>
    </div>
  </div>
{/if}

{#if editingSuite}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="w-full max-w-md rounded-2xl bg-white p-8 shadow-2xl dark:bg-gray-900"
    >
      <h2 class="text-xl font-black text-slate-800 dark:text-white">
        Edit Suite
      </h2>
      <div class="mt-6">
        <label
          for="edit-suite-name"
          class="block text-[12px] font-black uppercase tracking-widest text-slate-500"
          >Suite Name</label
        >
        <input
          id="edit-suite-name"
          type="text"
          bind:value={editingSuite.title}
          class="mt-3 w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-800 outline-none focus:border-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:focus:border-indigo-400"
          autofocus
          on:keydown={(e) => e.key === "Enter" && handleUpdateSuite()}
        />
      </div>
      <div class="mt-8 flex items-center justify-end gap-3">
        <button
          class="rounded-xl px-4 py-2 text-sm font-bold text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => (editingSuite = null)}>Cancel</button
        >
        <button
          class="rounded-xl bg-indigo-600 px-6 py-2 text-sm font-black text-white hover:bg-indigo-500 disabled:opacity-50"
          disabled={!editingSuite.title.trim()}
          on:click={handleUpdateSuite}>Save Changes</button
        >
      </div>
    </div>
  </div>
{/if}

{#if showDeleteSuiteModal && suiteToDelete}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="w-full max-w-md rounded-2xl bg-white p-8 shadow-2xl dark:bg-gray-900"
    >
      <h2 class="text-xl font-black text-slate-800 dark:text-white">
        Delete Suite
      </h2>
      <p class="mt-2 text-sm text-slate-500">
        Are you sure you want to delete <span class="font-bold text-indigo-500"
          >"{suiteToDelete.title}"</span
        >?
      </p>

      <div class="mt-6 space-y-3">
        <p
          class="text-[11px] font-black uppercase tracking-[0.2em] text-slate-400"
        >
          Handle associated cases
        </p>
        <button
          class="flex w-full items-center gap-3 rounded-xl border p-3 text-left transition-all {deleteMode ===
          'move'
            ? 'border-indigo-500 bg-indigo-500/5 ring-1 ring-indigo-500'
            : 'border-slate-200 hover:border-slate-300 dark:border-gray-700'}"
          on:click={() => (deleteMode = "move")}
        >
          <div
            class="grid h-8 w-8 shrink-0 place-items-center rounded-lg {deleteMode ===
            'move'
              ? 'bg-indigo-600 text-white'
              : 'bg-slate-100 text-slate-500 dark:bg-gray-800'}"
          >
            <Folder size={16} />
          </div>
          <div>
            <p
              class="text-sm font-bold {deleteMode === 'move'
                ? 'text-indigo-600 dark:text-indigo-400'
                : 'text-slate-700 dark:text-gray-200'}"
            >
              Move to Unassigned
            </p>
            <p class="text-[11px] text-slate-500">
              Keep test cases but remove them from this suite
            </p>
          </div>
        </button>

        <button
          class="flex w-full items-center gap-3 rounded-xl border p-3 text-left transition-all {deleteMode ===
          'delete'
            ? 'border-rose-500 bg-rose-500/5 ring-1 ring-rose-500'
            : 'border-slate-200 hover:border-slate-300 dark:border-gray-700'}"
          on:click={() => (deleteMode = "delete")}
        >
          <div
            class="grid h-8 w-8 shrink-0 place-items-center rounded-lg {deleteMode ===
            'delete'
              ? 'bg-rose-600 text-white'
              : 'bg-slate-100 text-slate-500 dark:bg-gray-800'}"
          >
            <Trash2 size={16} />
          </div>
          <div>
            <p
              class="text-sm font-bold {deleteMode === 'delete'
                ? 'text-rose-600 dark:text-rose-400'
                : 'text-slate-700 dark:text-gray-200'}"
            >
              Delete Everything
            </p>
            <p class="text-[11px] text-slate-500">
              Permanently remove this suite and all its test cases
            </p>
          </div>
        </button>
      </div>

      <div class="mt-8 flex items-center justify-end gap-3">
        <button
          class="rounded-xl px-4 py-2 text-sm font-bold text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => (showDeleteSuiteModal = false)}>Cancel</button
        >
        <button
          class="rounded-xl bg-rose-600 px-6 py-2 text-sm font-black text-white hover:bg-rose-500"
          on:click={handleDeleteSuite}>Delete Suite</button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 10px;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }

  :global(.property-select .property-trigger-btn) {
    background-color: transparent !important;
    border-width: 1px !important;
    border-style: solid !important;
    border-color: rgba(255, 255, 255, 0.1) !important;
    color: #9ca3af !important;
    border-radius: 9999px !important;
    height: 2rem !important;
    padding-left: 0.75rem !important;
    padding-right: 0.75rem !important;
    font-size: 13px !important;
    box-shadow: none !important;
  }

  :global(.property-select .property-trigger-btn:hover) {
    background-color: rgba(255, 255, 255, 0.05) !important;
    border-color: rgba(255, 255, 255, 0.2) !important;
  }

  :global(.property-select .property-trigger-btn span) {
    color: #d1d5db !important;
  }

  :global(.property-select .property-trigger-btn svg) {
    color: #6b7280 !important;
  }
</style>
