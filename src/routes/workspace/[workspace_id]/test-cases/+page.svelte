<script lang="ts">
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { currentWorkspaceName } from "$lib/stores/workspace";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { api, API_BASE_URL } from "$lib/apis";
  import { user } from "$lib/stores/auth";
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
    Save,
    FileDown,
    FileUp,
    Download,
    Trash2,
    X,
    Eye,
    CircleDashed,
    CheckCircle2,
    XCircle,
    Ban,
    History,
    AlertCircle,
    Check,
    Shield,
    Image as ImageIcon,
    ZoomIn,
    ZoomOut,
    Maximize2,
    RotateCw,
    ExternalLink,
    Paperclip,
    Copy,
    RotateCcw,
    FileCheck,
    LoaderCircle,
    MinusCircle,
    SignalHigh,
    SignalMedium,
    SignalLow,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import TestCaseStepList from "$lib/components/TestCaseStepList.svelte";
  import { getAssignees } from "$lib/db";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import { createWorkspacePageStore } from "$lib/stores/workspacePageStore";
  import WorkspaceModals from "$lib/components/WorkspaceModals.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import { createKeyboardHandler } from "$lib/stores/keyboardActions";
  import { getKeyboardConfig } from "$lib/stores/workspaceKeyboardConfig";
  import { theme } from "$lib/stores/theme";
  import { sprints } from "$lib/stores/sprintStore";
  import { buildMonthlySummary } from "$lib/utils/monthly-summary";
  import { modals } from "$lib/stores/uiActions";
  import { broadcastTestCaseAssignment } from "$lib/stores/testCaseNotifications";
  import { connectRealtime } from "$lib/stores/realtime";

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
    status: "draft" | "actual" | "failed" | "blocked" | "deprecated" | "passed";
    assignee?: string;
    description: string;
    preconditions?: string;
    postconditions?: string;
    steps: Step[];
    classic_steps?: ClassicStep[];
    gherkin_steps?: GherkinStep[];
    step_format: "classic" | "gherkin";
    input?: string;
    expected_result?: string;
    actual_result?: string;
    assign_dev?: string;
    fixed: "no" | "yes";
    suite_id: string;
    dev_note?: string;
    test_note?: string;
    attachments: TestCaseAttachment[];
  };

  type Suite = {
    id: string;
    title: string;
    description: string;
    cases: TestCase[];
    page: number;
    hasMore: boolean;
    totalCount?: number;
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

  type TestCaseAttachment = {
    id: string;
    filename: string;
    file_key: string;
    mime_type: string;
    size: number;
    uploaded_at: string;
  };

  type ClassicStep = {
    action: string;
    data: string;
    expected: string;
  };

  type TestRun = {
    _id: string;
    name: string;
    description: string | null;
    default_assignee: string;
    operating_system: string;
    test_cases: { test_case_id: string; status: string }[];
    created_at: string;
    status: "running" | "completed" | "aborted";
    created_by?: string;
    created_by_name?: string;
  };

  type TestRunCaseDetail = {
    test_case_id: string;
    status: string;
    test_case: {
      id: string;
      test_no: number;
      name: string;
      suite_id: string;
      priority: string;
      status: string;
    } | null;
  };

  type TestRunDetail = {
    id: string;
    workspace_id: string;
    name: string;
    description: string | null;
    default_assignee: string;
    operating_system: string;
    status: string;
    test_cases: TestRunCaseDetail[];
    stats: {
      total: number;
      pending: number;
      passed: number;
      failed: number;
      blocked: number;
      skipped: number;
      invalid: number;
    };
    created_by?: string;
    created_by_name?: string;
    created_by_avatar_url?: string;
    created_by_color?: string;
    created_at: string | null;
    updated_at: string | null;
  };

  type NewTestRunForm = {
    name: string;
    description: string;
    defaultAssignee: string;
    operatingSystem: string;
  };

  let suites: Suite[] = [];
  const ui = createUIActions();

  const ws = createWorkspacePageStore();
  const {
    assignees: wsAssignees,
    projectList,
    loadingData: wsLoadingData,
    workerStats,
    projectStats,
    workspaceActions: wsActions,
    taskActions,
    sprintActions,
    viewActions,
    searchInput,
    editingTask: wsEditingTask,
    allTasksIncludingArchived,
  } = ws;

  const keyboardHandler = createKeyboardHandler(
    getKeyboardConfig({
      uiActions: ui,
      modals,
      editingTask: $wsEditingTask,
      setEditingTask: (v) => ($wsEditingTask = v),
      searchInputRef: null,
      visibleTabs: [],
      viewActions,
    }),
  );

  function cancelTaskEdit() {
    $wsEditingTask = null;
    ui.closeModal("form");
  }

  function openTaskPage(task: any) {
    const wsId = $page.params.workspace_id;
    const urlRoom = $page.url.searchParams.get("room");
    const roomParam = urlRoom ? `?room=${urlRoom}` : "";
    goto(`${base}/workspace/${wsId}/task/${task.id}${roomParam}`);
  }

  $: isAuthorized =
    $user?.role === "admin" || $user?.profile?.position === "QA Tester";

  let query = "";
  let field = "all";
  let selectedSuiteId = "";
  let selectedCaseId = "";
  let workspaceId = $page.params.workspace_id;
  let showDetail = false;
  let detailPanelWidth = 420;
  let stopPanelResize: (() => void) | null = null;
  let leftSidebarWidth = 256;
  let stopSidebarResize: (() => void) | null = null;
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

  let showImportModal = false;
  let importProgress = { current: 0, total: 0, status: "" };
  let importSummary: {
    success: number;
    failed: number;
    failedNames: string[];
    skipped: { row: number; reason: string; name?: string }[];
  } | null = null;

  let quickTestInputSuiteId = "";
  let quickTestTitle = "";

  let sidebarSteps: any[] = [];
  let sidebarStepFormat: "classic" | "gherkin" = "classic";
  let isSavingSidebarSteps = false;
  let lastLoadedCaseId = "";
  let sidebarTab: "general" | "notes" | "properties" | "attachments" =
    "general";
  let isSavingNotes = false;
  let devNote = "";
  let testNote = "";

  let isEditingSidebarSteps = false;
  let showActionsMenu = false;

  let showFilterDropdown = false;
  let activeFilterProperty: string | null = null;
  let activeFilters: Record<string, string[]> = {
    priority: [],
    status: [],
    fixed: [],
    assign_dev: [],
  };

  let showDeleteModal = false;
  let caseToDeleteId = "";

  let testRuns: TestRun[] = [];
  let testRunsPage = 1;
  let testRunsTotal = 0;
  let testRunsLimit = 10;
  let isLoadingMoreRuns = false;
  let showNewTestRunModal = false;
  let viewMode: "suites" | "test-run" = "suites";
  let selectedRunDetail: TestRunDetail | null = null;
  let isLoadingRunDetail = false;
  let updatingCaseId: string | null = null;
  let openPriorityMenuId = "";
  let updatingPriorityCaseIds: Set<string> = new Set();
  let newTestRunForm: NewTestRunForm = {
    name: "",
    description: "",
    defaultAssignee: "unassigned",
    operatingSystem: "",
  };
  let selectedTestCaseIds: Set<string> = new Set();
  let testRunSearchQuery = "";
  let testRunSearchField = "all";
  let isTestRunsOpen = true;
  let isSuitesOpen = true;
  let showTestRunFilterDropdown = false;
  let testRunActiveFilterProperty: string | null = null;
  let testRunActiveFilters: Record<string, string[]> = {
    priority: [],
    status: [],
  };

  const testRunFilterProperties = [
    { id: "priority", label: "Priority" },
    { id: "status", label: "Status" },
  ];

  function toggleTestRunFilter(property: string, value: string) {
    if (!testRunActiveFilters[property]) testRunActiveFilters[property] = [];
    const idx = testRunActiveFilters[property].indexOf(value);
    if (idx >= 0) {
      testRunActiveFilters[property] = testRunActiveFilters[property].filter(
        (v) => v !== value,
      );
    } else {
      testRunActiveFilters[property] = [
        ...testRunActiveFilters[property],
        value,
      ];
    }
    testRunActiveFilters = { ...testRunActiveFilters };
  }

  $: hasTestRunFilters = Object.values(testRunActiveFilters).some(
    (v) => v.length > 0,
  );

  const osOptions = [
    { value: "", label: "Select value" },
    { value: "windows", label: "Windows" },
    { value: "macos", label: "macOS" },
    { value: "linux", label: "Linux" },
    { value: "ios", label: "iOS" },
    { value: "android", label: "Android" },
    { value: "other", label: "Other" },
  ];

  function getAutoTestRunName(): string {
    const now = new Date();
    const dd = String(now.getDate()).padStart(2, "0");
    const mm = String(now.getMonth() + 1).padStart(2, "0");
    const yyyy = now.getFullYear();
    return `Test run ${dd}/${mm}/${yyyy} (auto)`;
  }

  const RUN_CREATORS_KEY = "test-run-creators";

  function saveRunCreator(runId: string, createdBy: string, createdByName: string) {
    try {
      const raw = localStorage.getItem(RUN_CREATORS_KEY);
      const map = raw ? JSON.parse(raw) : {};
      map[runId] = { created_by: createdBy, created_by_name: createdByName };
      localStorage.setItem(RUN_CREATORS_KEY, JSON.stringify(map));
    } catch {}
  }

  function getRunCreator(runId: string): { created_by?: string; created_by_name?: string } {
    try {
      const raw = localStorage.getItem(RUN_CREATORS_KEY);
      const map = raw ? JSON.parse(raw) : {};
      return map[runId] ?? {};
    } catch {
      return {};
    }
  }

  function getCurrentUserDisplayName(): string {
    const u = $user;
    if (!u) return "";
    return u.profile?.nickname || u.profile?.first_name || u.email;
  }

  function openNewTestRunModal() {
    newTestRunForm = {
      name: getAutoTestRunName(),
      description: "",
      defaultAssignee: (() => {
        const me = assignees.find(
          (a) => String(a.user_id) === String($user?.id),
        );
        return me ? String(me.id) : "unassigned";
      })(),
      operatingSystem: "",
    };
    selectedTestCaseIds = new Set(
      suites.flatMap((s) => s.cases.map((c) => c.id)),
    );
    testRunSearchQuery = "";
    testRunSearchField = "all";
    testRunActiveFilters = { priority: [], status: [] };
    testRunActiveFilterProperty = null;
    showTestRunFilterDropdown = false;
    showNewTestRunModal = true;
  }

  function toggleTestCaseSelection(id: string) {
    const next = new Set(selectedTestCaseIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedTestCaseIds = next;
  }

  function toggleSuiteSelection(suite: Suite) {
    const allSelected = suite.cases.every((c) => selectedTestCaseIds.has(c.id));
    const next = new Set(selectedTestCaseIds);
    suite.cases.forEach((c) =>
      allSelected ? next.delete(c.id) : next.add(c.id),
    );
    selectedTestCaseIds = next;
  }

  async function handleCreateTestRun() {
    if (!newTestRunForm.name.trim() || !workspaceId) return;
    try {
      const res = await api.testRuns.create(workspaceId, {
        name: newTestRunForm.name.trim(),
        description: newTestRunForm.description.trim() || undefined,
        default_assignee: newTestRunForm.defaultAssignee,
        operating_system: newTestRunForm.operatingSystem,
        test_case_ids: Array.from(selectedTestCaseIds),
      });
      if (!res.ok) throw new Error(await res.text());
      const created: TestRun = await res.json();
      const creatorName = getCurrentUserDisplayName();
      const creatorId = $user?.id ?? "";
      saveRunCreator(created._id, creatorId, creatorName);
      testRuns = [created, ...testRuns];
      testRunsTotal++;
      showNewTestRunModal = false;
      isSuitesOpen = false;
      openTestRun(created);
      ui.showMessage("Test run created", "success");
    } catch (e) {
      ui.showMessage("Failed to create test run", "error");
    }
  }

  async function loadTestRunsFromStorage() {
    if (!workspaceId) return;
    try {
      testRunsPage = 1;
      const res = await api.testRuns.list(workspaceId, 1, testRunsLimit);
      if (res.ok) {
        const body = await res.json();
        testRuns = body.data;
        testRunsTotal = body.total;
      }
    } catch {
      testRuns = [];
      testRunsTotal = 0;
    }
  }

  async function loadMoreTestRuns() {
    if (!workspaceId || isLoadingMoreRuns) return;
    isLoadingMoreRuns = true;
    try {
      const nextPage = testRunsPage + 1;
      const res = await api.testRuns.list(workspaceId, nextPage, testRunsLimit);
      if (res.ok) {
        const body = await res.json();
        testRuns = [...testRuns, ...body.data];
        testRunsTotal = body.total;
        testRunsPage = nextPage;
      }
    } catch {
      // ignore
    } finally {
      isLoadingMoreRuns = false;
    }
  }

  function resolveCreator(userId: string): { name: string; avatarUrl?: string; color: string } {
    const match = assignees.find((a) => String(a.user_id) === String(userId));
    return {
      name: match?.name ?? userId,
      avatarUrl: match?.avatar_url || match?.avatarUrl || undefined,
      color: match?.color || "#6366f1",
    };
  }

  async function openTestRun(run: TestRun) {
    if (!workspaceId) return;
    isLoadingRunDetail = true;
    viewMode = "test-run";
    selectedRunDetail = null;
    try {
      const res = await api.testRuns.get(workspaceId, run._id);
      if (res.ok) {
        const detail = await res.json();
        const localCreator = getRunCreator(run._id);
        const createdBy = detail.created_by || localCreator.created_by;
        if (createdBy) {
          const info = resolveCreator(createdBy);
          selectedRunDetail = {
            ...detail,
            created_by: createdBy,
            created_by_name: info.name,
            created_by_avatar_url: info.avatarUrl,
            created_by_color: info.color,
          };
        } else {
          selectedRunDetail = {
            ...detail,
            created_by_name: localCreator.created_by_name,
          };
        }
      }
    } catch {}
    isLoadingRunDetail = false;
  }

  function closeTestRun() {
    viewMode = "suites";
    selectedRunDetail = null;
  }

  async function updateRunCaseStatus(tcId: string, status: string) {
    if (!workspaceId || !selectedRunDetail) return;
    updatingCaseId = tcId;
    try {
      const res = await api.testRuns.updateCaseStatus(
        workspaceId,
        selectedRunDetail.id,
        tcId,
        status,
      );
      if (res.ok) {
        selectedRunDetail = {
          ...selectedRunDetail,
          test_cases: selectedRunDetail.test_cases.map((c) =>
            c.test_case_id === tcId ? { ...c, status } : c,
          ),
        };
        // recompute stats locally
        const counts = {
          pending: 0,
          passed: 0,
          failed: 0,
          blocked: 0,
          skipped: 0,
          invalid: 0,
        };
        for (const c of selectedRunDetail.test_cases) {
          const k = c.status as keyof typeof counts;
          if (k in counts) counts[k]++;
          else counts.pending++;
        }
        selectedRunDetail = {
          ...selectedRunDetail,
          stats: { total: selectedRunDetail.test_cases.length, ...counts },
        };

        // Sync main test case status on pass/fail
        // Map test run statuses to test case statuses: "passed" → "passed", "failed" → "failed"
        if (status === "passed" || status === "failed") {
          const tcStatus = status === "passed" ? "passed" : "failed";
          const tcRes = await api.data.testCases.updateStatus(tcId, tcStatus);
          if (tcRes.ok) {
            suites = suites.map((s) => ({
              ...s,
              cases: s.cases.map((c) =>
                c.id === tcId ? { ...c, status: tcStatus as any } : c,
              ),
            }));
          }
        }
      }
    } catch {}
    updatingCaseId = null;
  }

  async function updateRunStatus(status: string) {
    if (!workspaceId || !selectedRunDetail) return;
    const res = await api.testRuns.updateStatus(
      workspaceId,
      selectedRunDetail.id,
      status,
    );
    if (res.ok) {
      selectedRunDetail = { ...selectedRunDetail, status };
      testRuns = testRuns.map((r) =>
        r._id === selectedRunDetail!.id ? { ...r, status: status as any } : r,
      );
    }
  }

  let showDeleteRunModal = false;
  let runToDelete: TestRun | null = null;
  let suiteMenuId: string | null = null;
  let runMenuId: string | null = null;
  let contextMenuX = 0;
  let contextMenuY = 0;

  function openContextMenu(e: MouseEvent, type: "suite" | "run", id: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    contextMenuX = rect.right + 2;
    contextMenuY = rect.top - 4;
    if (type === "suite") suiteMenuId = id;
    else runMenuId = id;
  }

  async function confirmDeleteTestRun() {
    const target =
      runToDelete ??
      (selectedRunDetail ? ({ _id: selectedRunDetail.id } as any) : null);
    if (!workspaceId || !target) return;
    const res = await api.testRuns.delete(workspaceId, target._id);
    if (res.ok) {
      testRuns = testRuns.filter((r) => r._id !== target._id);
      testRunsTotal = Math.max(0, testRunsTotal - 1);
      if (selectedRunDetail?.id === target._id) closeTestRun();
      showDeleteRunModal = false;
      runToDelete = null;
      ui.showMessage("Test run deleted", "success");
    } else {
      ui.showMessage("Failed to delete test run", "error");
    }
  }

  const RUN_CASE_STATUSES = [
    {
      value: "pending",
      label: "Pending",
      icon: CircleDashed,
      iconClass: "text-slate-400",
    },
    {
      value: "passed",
      label: "Passed",
      icon: CheckCircle2,
      iconClass: "text-emerald-500",
    },
    {
      value: "failed",
      label: "Failed",
      icon: XCircle,
      iconClass: "text-red-500",
    },
    {
      value: "blocked",
      label: "Blocked",
      icon: Ban,
      iconClass: "text-orange-500",
    },
    {
      value: "skipped",
      label: "Skipped",
      icon: MinusCircle,
      iconClass: "text-blue-400",
    },
    {
      value: "invalid",
      label: "Invalid",
      icon: AlertCircle,
      iconClass: "text-purple-500",
    },
  ];

  let editingField: string | null = null;
  let tempFieldValue = "";
  let isSavingField = false;

  function startEditing(field: string, value: string) {
    if (!isAuthorized) return;
    editingField = field;
    tempFieldValue = value || "";
  }

  async function saveField(field: string) {
    if (!selectedCase || !workspaceId) return;
    isSavingField = true;
    try {
      const dbField = field === "title" ? "name" : field;
      const resp = await api.data.testCases.update(selectedCase.id, {
        [dbField]: tempFieldValue,
      });
      if (resp.ok) {
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === selectedCase.id ? { ...c, [field]: tempFieldValue } : c,
          ),
        }));
        editingField = null;
        ui.showMessage("Field updated", "success");
      } else {
        ui.showMessage("Failed to update field", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    } finally {
      isSavingField = false;
    }
  }

  const filterProperties = [
    { id: "priority", label: "Priority" },
    { id: "status", label: "Status" },
    { id: "fixed", label: "Fixed" },
    { id: "assign_dev", label: "Assign Dev" },
  ];

  function toggleFilterValue(property: string, value: string) {
    if (!activeFilters[property]) {
      activeFilters[property] = [];
    }
    const idx = activeFilters[property].indexOf(value);
    if (idx >= 0) {
      activeFilters[property] = activeFilters[property].filter(
        (v) => v !== value,
      );
    } else {
      activeFilters[property] = [...activeFilters[property], value];
    }
    // Update API or re-fetch data
    loadData(query, field);
  }

  function getFilterLabel(prop: string, value: string) {
    if (prop === "priority")
      return priorityOptions.find((o) => o.value === value)?.label || value;
    if (prop === "status")
      return statusOptions.find((o) => o.value === value)?.label || value;
    if (prop === "fixed")
      return fixedOptions.find((o) => o.value === value)?.label || value;
    if (prop === "assign_dev") {
      const opt = availableAssigneeOptions.find((o) => o.value === value);
      return opt ? opt.label : value;
    }
    return value;
  }

  async function deleteTestCase(id: string) {
    caseToDeleteId = id;
    showDeleteModal = true;
  }

  async function confirmDeleteTestCase() {
    if (!caseToDeleteId) return;
    try {
      const resp = await api.data.testCases.delete(caseToDeleteId);
      if (resp.ok) {
        suites = suites.map((s) => {
          const filtered = s.cases.filter((c) => c.id !== caseToDeleteId);
          const removed = filtered.length < s.cases.length;
          return {
            ...s,
            cases: filtered,
            totalCount: removed
              ? Math.max(0, (s.totalCount ?? s.cases.length) - 1)
              : (s.totalCount ?? s.cases.length),
          };
        });
        if (selectedCaseId === caseToDeleteId) {
          selectedCaseId = "";
        }
        ui.showMessage("Test case deleted", "success");
      } else {
        ui.showMessage("Failed to delete test case", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    } finally {
      showDeleteModal = false;
      caseToDeleteId = "";
    }
  }

  async function handleConvertToTask(id: string) {
    try {
      const resp = await api.data.testCases.convertToTask(id);
      if (resp.ok) {
        const data = await resp.json();
        ui.showMessage("Task created successfully", "success");
        // Redirect to task detail
        const urlRoom =
          $page.url.searchParams.get("room") ||
          (browser ? localStorage.getItem("sync-room-code") : null);
        const roomParam = urlRoom ? `?room=${urlRoom}` : "";
        goto(
          `${base}/workspace/${workspaceId}/task/${data.task_id}${roomParam}`,
        );
      } else {
        const error = await resp.text();
        ui.showMessage(`Failed to create task: ${error}`, "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    }
  }

  function focusOnInit(node: HTMLElement) {
    node.focus();
  }

  // Lightbox state
  let lightboxImages: { src: string; alt: string }[] = [];
  let lightboxSrc = "";
  let lightboxAlt = "";
  let lightboxOpen = false;
  let lightboxZoom = 1;
  let lightboxX = 0;
  let lightboxY = 0;
  let lightboxRotation = 0;
  let lightboxIndex = 0;
  let isLightboxDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartOffsetX = 0;
  let dragStartOffsetY = 0;
  let lightboxCopyFeedback = "";

  function getAttachmentUrl(fileKey: string): string {
    return `${API_BASE_URL.replace(/\/api$/, "")}/api/files/${fileKey}`;
  }

  function openLightbox(
    images: { file_key: string; filename?: string }[],
    index: number,
  ) {
    lightboxImages = images.map((img) => ({
      src: getAttachmentUrl(img.file_key),
      alt: img.filename || "Image",
    }));
    lightboxIndex = Math.min(Math.max(index, 0), lightboxImages.length - 1);
    lightboxSrc = lightboxImages[lightboxIndex].src;
    lightboxAlt = lightboxImages[lightboxIndex].alt;
    lightboxZoom = 1;
    lightboxX = 0;
    lightboxY = 0;
    lightboxRotation = 0;
    lightboxCopyFeedback = "";
    lightboxOpen = true;
  }

  function closeLightbox() {
    lightboxOpen = false;
    isLightboxDragging = false;
  }

  function lbZoomIn() {
    lightboxZoom = Math.min(lightboxZoom + 0.25, 5);
  }
  function lbZoomOut() {
    lightboxZoom = Math.max(lightboxZoom - 0.25, 0.25);
  }
  function lbResetView() {
    lightboxZoom = 1;
    lightboxX = 0;
    lightboxY = 0;
    lightboxRotation = 0;
  }
  function lbRotateRight() {
    lightboxRotation = (lightboxRotation + 90) % 360;
  }

  function lbNavigatePrev() {
    if (lightboxImages.length <= 1) return;
    lightboxIndex =
      (lightboxIndex - 1 + lightboxImages.length) % lightboxImages.length;
    lightboxSrc = lightboxImages[lightboxIndex].src;
    lightboxAlt = lightboxImages[lightboxIndex].alt;
    lbResetView();
  }

  function lbNavigateNext() {
    if (lightboxImages.length <= 1) return;
    lightboxIndex = (lightboxIndex + 1) % lightboxImages.length;
    lightboxSrc = lightboxImages[lightboxIndex].src;
    lightboxAlt = lightboxImages[lightboxIndex].alt;
    lbResetView();
  }

  function lbOpenInNewTab() {
    window.open(lightboxSrc, "_blank", "noopener,noreferrer");
  }

  async function lbDownload() {
    try {
      const a = document.createElement("a");
      const res = await fetch(lightboxSrc);
      const blob = await res.blob();
      a.href = URL.createObjectURL(blob);
      a.download =
        (lightboxAlt && lightboxAlt !== "Image" ? lightboxAlt : "image") +
        ".png";
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
    } catch {}
  }

  function handleLightboxMouseDown(event: MouseEvent) {
    if (event.button !== 0) return;
    isLightboxDragging = true;
    dragStartX = event.clientX;
    dragStartY = event.clientY;
    dragStartOffsetX = lightboxX;
    dragStartOffsetY = lightboxY;
  }

  function handleLightboxMouseMove(event: MouseEvent) {
    if (!isLightboxDragging) return;
    lightboxX = dragStartOffsetX + (event.clientX - dragStartX);
    lightboxY = dragStartOffsetY + (event.clientY - dragStartY);
  }

  function handleLightboxMouseUp() {
    isLightboxDragging = false;
  }

  function handleLightboxKeydown(event: KeyboardEvent) {
    if (!lightboxOpen) return;
    switch (event.key) {
      case "Escape":
        closeLightbox();
        break;
      case "+":
      case "=":
        lbZoomIn();
        break;
      case "-":
        lbZoomOut();
        break;
      case "0":
        lbResetView();
        break;
      case "r":
      case "R":
        lbRotateRight();
        break;
      case "ArrowLeft":
        lbNavigatePrev();
        break;
      case "ArrowRight":
        lbNavigateNext();
        break;
    }
  }

  $: if (selectedCase && selectedCase.id !== lastLoadedCaseId) {
    lastLoadedCaseId = selectedCase.id;
    isEditingSidebarSteps = false;
    sidebarStepFormat = selectedCase.step_format || "classic";
    sidebarSteps =
      sidebarStepFormat === "gherkin"
        ? [...(selectedCase.gherkin_steps || [])]
        : [...(selectedCase.classic_steps || selectedCase.steps || [])];
    devNote = selectedCase.dev_note || "";
    testNote = selectedCase.test_note || "";
  }

  async function saveSidebarSteps() {
    if (!selectedCase || !workspaceId) return;
    isSavingSidebarSteps = true;
    try {
      const payload = {
        step_format: sidebarStepFormat,
        classic_steps:
          sidebarStepFormat === "classic" ? sidebarSteps : undefined,
        gherkin_steps:
          sidebarStepFormat === "gherkin" ? sidebarSteps : undefined,
      };
      const resp = await api.data.testCases.updateSteps(
        selectedCase.id,
        payload,
      );
      if (resp.ok) {
        isEditingSidebarSteps = false;
        ui.showMessage("Steps updated", "success");
        // Update local suites state
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === selectedCase.id
              ? {
                  ...c,
                  step_format: sidebarStepFormat,
                  classic_steps:
                    sidebarStepFormat === "classic" ? sidebarSteps : undefined,
                  gherkin_steps:
                    sidebarStepFormat === "gherkin" ? sidebarSteps : undefined,
                  steps:
                    sidebarStepFormat === "classic"
                      ? sidebarSteps
                      : sidebarSteps.map((gs) => ({
                          action: `${gs.keyword} ${gs.text}`,
                          data: "",
                          expected: "",
                        })),
                }
              : c,
          ),
        }));
      } else {
        ui.showMessage("Failed to update steps", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    } finally {
      isSavingSidebarSteps = false;
    }
  }

  async function handleUpdateNotes() {
    if (!selectedCase) return;
    isSavingNotes = true;
    try {
      const resp = await api.data.testCases.updateNotes(selectedCase.id, {
        dev_note: devNote,
        test_note: testNote,
      });
      if (resp.ok) {
        ui.showMessage("Notes updated", "success");
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === selectedCase.id
              ? { ...c, dev_note: devNote, test_note: testNote }
              : c,
          ),
        }));
      } else {
        ui.showMessage("Failed to update notes", "error");
      }
    } catch (e) {
      ui.showMessage("An error occurred", "error");
    } finally {
      isSavingNotes = false;
    }
  }

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
    const suiteId = editingSuite.id || (editingSuite as any)._id;
    const newTitle = editingSuite.title;

    console.log("Updating suite:", { suiteId, newTitle });

    try {
      const resp = await api.data.testSuites.update(suiteId, {
        title: newTitle,
      });
      if (resp.ok) {
        // Force a new array reference to ensure Svelte reactivity
        suites = suites.map((s) => {
          const currentId = s.id || (s as any)._id;
          if (currentId === suiteId) {
            return { ...s, title: newTitle };
          }
          return s;
        });

        ui.showMessage("Suite updated", "success");
        editingSuite = null;
      } else {
        const errorData = await resp.text();
        console.error(
          "Failed to update suite. Status:",
          resp.status,
          "Error:",
          errorData,
        );
        ui.showMessage(`Failed to update suite: ${errorData}`, "error");
      }
    } catch (e) {
      console.error("Update suite error:", e);
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
      step_format: tc.step_format || "classic",
      classic_steps: tc.classic_steps,
      gherkin_steps: tc.gherkin_steps,
      steps:
        tc.classic_steps ||
        (tc.gherkin_steps || []).map((s: any) => ({
          action: `${s.keyword} ${s.text}`,
          data: "",
          expected: "",
        })),
      input: tc.input || "",
      expected_result: tc.expected_result || "",
      actual_result: tc.actual_result || "",
      assign_dev: tc.assign_dev || "unassigned",
      fixed: tc.fixed || "no",
      suite_id: tc.suite_id,
      dev_note: tc.dev_note || "",
      test_note: tc.test_note || "",
      attachments: tc.attachments || [],
    };
  }

  async function updateStatus(testCase: TestCase, newStatus: string) {
    try {
      const resp = await api.data.testCases.updateStatus(
        testCase.id,
        newStatus,
      );
      if (resp.ok) {
        // Update local state
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, status: newStatus as any } : c,
          ),
        }));
        ui.showMessage("Status updated", "success");
      }
    } catch (e) {
      ui.showMessage("Failed to update status", "error");
    }
  }

  async function updateFixed(testCase: TestCase, newFixed: string) {
    try {
      const resp = await api.data.testCases.updateFixed(testCase.id, newFixed);
      if (resp.ok) {
        // Update local state
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, fixed: newFixed as any } : c,
          ),
        }));
        ui.showMessage("Fixed status updated", "success");
      } else {
        ui.showMessage("Failed to update fixed status", "error");
      }
    } catch (e) {
      ui.showMessage("Failed to update fixed status", "error");
    }
  }

  async function updatePriority(testCase: TestCase, newPriority: string) {
    const priority = newPriority as TestCase["priority"];
    if (testCase.priority === priority) {
      openPriorityMenuId = "";
      return;
    }

    const previousPriority = testCase.priority;
    openPriorityMenuId = "";
    updatingPriorityCaseIds = new Set(updatingPriorityCaseIds).add(testCase.id);
    patchTestCase(testCase.id, { priority });

    try {
      const resp = await api.data.testCases.updatePriority(
        testCase.id,
        priority,
      );
      if (resp.ok) {
        ui.showMessage("Priority updated", "success");
      } else {
        patchTestCase(testCase.id, { priority: previousPriority });
        ui.showMessage("Failed to update priority", "error");
      }
    } catch (e) {
      patchTestCase(testCase.id, { priority: previousPriority });
      ui.showMessage("Failed to update priority", "error");
    } finally {
      const next = new Set(updatingPriorityCaseIds);
      next.delete(testCase.id);
      updatingPriorityCaseIds = next;
    }
  }

  async function updateAssignTester(
    testCase: TestCase,
    newAssignTester: string,
  ) {
    try {
      const resp = await api.data.testCases.updateAssignTester(
        testCase.id,
        newAssignTester,
      );
      if (resp.ok) {
        const previousAssignee = testCase.assignee || "unassigned";
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, assignee: newAssignTester } : c,
          ),
        }));
        broadcastTestCaseAssignment({
          id: testCase.id,
          title: testCase.title,
          workspaceId: workspaceId!,
          suiteId: testCase.suite_id,
          assignees,
          nextAssigneeIds: [newAssignTester],
          previousAssigneeIds: [previousAssignee],
          action: "update",
        });
        ui.showMessage("Tester updated", "success");
      } else {
        ui.showMessage("Failed to update tester", "error");
      }
    } catch (e) {
      ui.showMessage("Failed to update tester", "error");
    }
  }

  async function updateAssignDev(testCase: TestCase, newAssignDev: string) {
    try {
      const resp = await api.data.testCases.updateAssignDev(
        testCase.id,
        newAssignDev,
      );
      if (resp.ok) {
        const previousAssignee = testCase.assign_dev || "unassigned";
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, assign_dev: newAssignDev } : c,
          ),
        }));
        broadcastTestCaseAssignment({
          id: testCase.id,
          title: testCase.title,
          workspaceId: workspaceId!,
          suiteId: testCase.suite_id,
          assignees,
          nextAssigneeIds: [newAssignDev],
          previousAssigneeIds: [previousAssignee],
          action: "update",
        });
        ui.showMessage("Assignee updated", "success");
      } else {
        const err = await resp.text();
        ui.showMessage(`Failed to update assignee: ${err}`, "error");
      }
    } catch (e) {
      ui.showMessage("Failed to update assignee", "error");
    }
  }

  async function loadData(
    q?: string,
    searchField?: string,
    keepState: boolean = false,
  ) {
    if (!workspaceId) return;
    try {
      // Store current state if needed
      const prevPages = new Map(suites.map((s) => [s.id, s.page]));
      const prevSelectedSuiteId = selectedSuiteId;
      const prevSelectedCaseId = selectedCaseId;

      const filterParams: Record<string, string | undefined> = {
        q: q || query || undefined,
        field: searchField || field || undefined,
        priority:
          activeFilters.priority?.length > 0
            ? activeFilters.priority.join(",")
            : undefined,
        status:
          activeFilters.status?.length > 0
            ? activeFilters.status.join(",")
            : undefined,
        fixed:
          activeFilters.fixed?.length > 0
            ? activeFilters.fixed.join(",")
            : undefined,
        assign_dev:
          activeFilters.assign_dev?.length > 0
            ? activeFilters.assign_dev.join(",")
            : undefined,
      };

      // Fetch suites (includes case_count and unassigned)
      const suiteResp = await api.data.testSuites.list(workspaceId);
      if (!suiteResp.ok) return;
      const suiteData = await suiteResp.json();

      const suiteList = suiteData.map((s: any) => ({
        ...s,
        id: s.id || s._id,
        totalCount: s.case_count ?? 0,
      }));

      // Load cases per-suite in parallel
      const casesPromises = suiteList.map((s: any) => {
        const page = keepState ? prevPages.get(s.id) || 1 : 1;
        return api.data.testCases.list(workspaceId!, {
          suite_id: s.id === "unassigned" ? "none" : s.id,
          page: page,
          limit: 10,
          ...Object.fromEntries(
            Object.entries(filterParams).filter(([_, v]) => v !== undefined),
          ),
        });
      });

      const casesResponses = await Promise.all(casesPromises);

      let newSuites = [];
      for (let i = 0; i < suiteList.length; i++) {
        const s = suiteList[i];
        const page = keepState ? prevPages.get(s.id) || 1 : 1;
        let cases: any[] = [];
        if (casesResponses[i].ok) {
          const data = await casesResponses[i].json();
          cases = data.map(mapBackendToFrontend);
        }
        newSuites.push({
          ...s,
          cases,
          page,
          hasMore: cases.length >= 10,
        });
      }

      suites = newSuites;

      if (suites.length > 0) {
        const suiteExists = suites.some((s) => s.id === prevSelectedSuiteId);
        if (!keepState || !prevSelectedSuiteId || !suiteExists) {
          selectedSuiteId = suites[0].id;
          testCaseForm.suiteId =
            suites[0].id === "unassigned" ? "" : suites[0].id;
          if (suites[0].cases.length > 0) {
            selectedCaseId = suites[0].cases[0].id;
          }
        } else {
          selectedSuiteId = prevSelectedSuiteId;
          const currentSuite = suites.find((s) => s.id === selectedSuiteId);
          const caseExists = currentSuite?.cases.some(
            (c) => c.id === prevSelectedCaseId,
          );
          if (caseExists) {
            selectedCaseId = prevSelectedCaseId;
          } else if (currentSuite && currentSuite.cases.length > 0) {
            selectedCaseId = currentSuite.cases[0].id;
          }
        }
      }
    } catch (e) {
      console.error("Failed to load test cases:", e);
    }
  }

  let searchTimeout: any;
  $: if ((query !== undefined || field !== undefined) && browser) {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      loadData(query, field);
    }, 300);
  }

  $: monthlySummary = buildMonthlySummary($allTasksIncludingArchived);

  let refreshInterval: any;

  onMount(async () => {
    document.addEventListener("keydown", keyboardHandler);
    assignees = await getAssignees(true);
    const roomCode =
      $page.url.searchParams.get("room") ||
      (browser ? localStorage.getItem("sync-room-code") : null);
    if (roomCode) {
      connectRealtime(roomCode, (payload) => {
        if (payload.entity === "test_case") {
          void loadData(undefined, undefined, true);
        }
      });
    }
    await loadData();
    ws.loadData();
    loadTestRunsFromStorage();

    // Auto refresh every 5 minutes (300,000 ms)
    refreshInterval = setInterval(() => {
      loadData(undefined, undefined, true);
    }, 300000);
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("keydown", keyboardHandler);
    }
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
  function downloadTemplate() {
    const headers = [
      "Suite",
      "Test Case No",
      "Test Name",
      "Status",
      "Assign Tester",
      "Assign Dev",
      "Precondition",
      "Input",
      "Expected Result",
      "Actual Result",
      "Test Step",
    ];
    const rows = [
      [
        "Authentication",
        "1",
        "Sample Login",
        "draft",
        "unassigned",
        "unassigned",
        "User is on login page",
        "email@example.com, password123",
        "User redirected to dashboard",
        "User redirected to dashboard",
        "Enter email\nEnter password\nClick login",
      ],
    ];

    let csv = headers.join(",") + "\n";
    rows.forEach((row) => {
      csv += row.map((v) => `"${v.replace(/"/g, '""')}"`).join(",") + "\n";
    });

    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    const url = URL.createObjectURL(blob);
    link.setAttribute("href", url);
    link.setAttribute("download", "test_case_template.csv");
    link.style.visibility = "hidden";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  async function exportToCSV() {
    if (!workspaceId) return;
    try {
      ui.showMessage("Preparing export...", "info");

      // Fetch all users to get positions
      const usersResp = await api.auth.listUsers(workspaceId);
      const usersData = await usersResp.json();
      const userPositionMap = new Map<string, string>();
      const userNameMap = new Map<string, string>();

      if (usersData.success && Array.isArray(usersData.users)) {
        usersData.users.forEach((u: any) => {
          const id = u.id || u.user_id;
          userNameMap.set(
            id,
            u.profile?.nickname || u.profile?.first_name || u.email,
          );
          userPositionMap.set(id, u.profile?.position || "");
        });
      }

      // Fetch all cases in workspace
      const casesResp = await api.data.testCases.all(workspaceId);
      if (!casesResp.ok) {
        ui.showMessage("Failed to fetch test cases for export", "error");
        return;
      }
      const allCasesData = await casesResp.json();
      const allCases = allCasesData.map(mapBackendToFrontend);

      // Fetch all suites for titles
      const suitesResp = await api.data.testSuites.list(workspaceId);
      const suitesData = await suitesResp.json();
      const suiteMap = new Map<string, string>(
        suitesData.map((s: any) => [s.id || s._id, String(s.title || "")]),
      );
      suiteMap.set("unassigned", "Unassigned");

      const headers = [
        "Suite",
        "Test Case No",
        "Test Name",
        "Status",
        "Assign Tester",
        "Tester Position",
        "Assign Dev",
        "Dev Position",
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
          steps = (c.classic_steps || c.steps || [])
            .map((s: any) => s.action)
            .join("\n");
        }

        const testerId = c.assignee || "unassigned";
        const devId = c.assign_dev || "unassigned";

        rows.push([
          String(suiteMap.get(c.suite_id) || "Unassigned"),
          String(c.test_no),
          String(c.title || ""),
          String(c.status || ""),
          String(
            userNameMap.get(testerId) ||
              (testerId === "unassigned"
                ? $_("testCase__unassigned")
                : testerId),
          ),
          String(userPositionMap.get(testerId) || ""),
          String(
            userNameMap.get(devId) ||
              (devId === "unassigned" ? $_("testCase__unassigned") : devId),
          ),
          String(userPositionMap.get(devId) || ""),
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
      link.setAttribute(
        "download",
        `test_cases_export_${new Date().toISOString().split("T")[0]}.csv`,
      );
      link.style.visibility = "hidden";
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      ui.showMessage("Export complete", "success");
    } catch (e) {
      console.error("Export error:", e);
      ui.showMessage("Failed to export test cases", "error");
    }
  }

  function parseCSV(text: string): string[][] {
    const rows: string[][] = [];
    let currentRow: string[] = [];
    let currentField = "";
    let inQuotes = false;

    // Normalize line endings to \n
    const normalizedText = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");

    for (let i = 0; i < normalizedText.length; i++) {
      const char = normalizedText[i];
      const nextChar = normalizedText[i + 1];

      if (char === '"') {
        if (inQuotes && nextChar === '"') {
          currentField += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (char === "," && !inQuotes) {
        currentRow.push(currentField);
        currentField = "";
      } else if (char === "\n" && !inQuotes) {
        currentRow.push(currentField);
        rows.push(currentRow);
        currentRow = [];
        currentField = "";
      } else {
        currentField += char;
      }
    }

    if (currentField || currentRow.length > 0) {
      currentRow.push(currentField);
      rows.push(currentRow);
    }

    // Filter out rows that are entirely empty
    return rows.filter((r) => r.some((field) => field.trim().length > 0));
  }

  async function handleImport(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    const reader = new FileReader();

    reader.onload = async (e) => {
      const text = e.target?.result as string;
      const rows = parseCSV(text);
      if (rows.length <= 1) {
        ui.showMessage(
          "CSV file appears to be empty or has only headers.",
          "error",
        );
        return;
      }

      const headers = rows[0].map((h, i) => {
        let cleaned = h.trim().toLowerCase();
        if (i === 0) cleaned = cleaned.replace(/^\uFEFF/, "");
        return cleaned;
      });

      const casesToCreate = [];

      // Mapping for suite titles to IDs
      const suiteMap = new Map<string, string>();
      suites.forEach((s) => {
        if (s.id !== "unassigned") {
          suiteMap.set(s.title.toLowerCase().trim(), s.id);
        }
      });

      const skipped: { row: number; reason: string }[] = [];

      for (let i = 1; i < rows.length; i++) {
        const values = rows[i];
        if (values.length === 0) continue;

        const row: Record<string, string> = {};
        headers.forEach((h, idx) => {
          row[h] = (values[idx] || "").trim();
        });

        const title =
          row["test name"] ||
          row["title"] ||
          row["test name "] ||
          row["name"] ||
          row["testname"];

        if (!title) {
          skipped.push({
            row: i + 1,
            reason: 'Missing "Test Name" column or value.',
          });
          continue;
        }

        // Handle Suite logic
        let suiteId: string | null =
          selectedSuiteId === "unassigned" ? null : selectedSuiteId;
        const suiteName =
          row["suite"] || row["collection"] || row["test suite"] || row["suit"];

        if (suiteName && suiteName.trim()) {
          const normalizedName = suiteName.trim().toLowerCase();
          if (suiteMap.has(normalizedName)) {
            suiteId = suiteMap.get(normalizedName) || null;
          } else {
            // Create new suite
            try {
              if (workspaceId) {
                const sResp = await api.data.testSuites.create(workspaceId, {
                  title: suiteName.trim(),
                });
                if (sResp.ok) {
                  const newS = await sResp.json();
                  const newSId = newS.id || newS._id;
                  suiteMap.set(normalizedName, newSId);
                  suiteId = newSId;
                  suites = [
                    ...suites,
                    {
                      ...newS,
                      id: newSId,
                      cases: [],
                      page: 1,
                      hasMore: false,
                    },
                  ];
                }
              }
            } catch (err) {
              console.error("Failed to create suite:", err);
            }
          }
        }

        const stepsStr =
          row["test step"] ||
          row["steps"] ||
          row["step"] ||
          row["test steps"] ||
          "";
        const classicSteps = stepsStr
          .split("\n")
          .map((s) => s.trim())
          .filter((s) => s)
          .map((s) => ({
            action: s,
            data: "",
            expected: "",
          }));

        casesToCreate.push({
          name: title,
          description: row["description"] || "",
          preconditions: row["precondition"] || row["pre-condition"] || "",
          postconditions: row["postcondition"] || row["post-condition"] || "",
          input: row["input"] || "",
          expected_result: row["expected result"] || row["expected"] || "",
          actual_result: row["actual result"] || row["actual"] || "",
          status: row["status"] || "actual",
          priority: row["priority"] || "medium",
          fixed: row["fixed"] || "no",
          assign_dev: row["assign dev"] || row["developer"] || "unassigned",
          assign_tester: row["assign tester"] || row["tester"] || "unassigned",
          step_format: "classic",
          classic_steps: classicSteps.length > 0 ? classicSteps : undefined,
          suite_id: suiteId,
        });
      }

      if (casesToCreate.length > 0) {
        showImportModal = true;
        importProgress = {
          current: 0,
          total: casesToCreate.length,
          status: "Starting import...",
        };

        let successCount = 0;
        let failedCount = 0;
        const failedNames: string[] = [];

        for (let i = 0; i < casesToCreate.length; i++) {
          const tc = casesToCreate[i];
          importProgress = {
            current: i + 1,
            total: casesToCreate.length,
            status: `Creating: ${tc.name}`,
          };

          try {
            if (!workspaceId) continue;
            const resp = await api.data.testCases.create(workspaceId, tc);
            if (resp.ok) {
              successCount++;
            } else {
              failedCount++;
              failedNames.push(tc.name);
              console.error(`Failed "${tc.name}":`, await resp.text());
            }
          } catch (err) {
            failedCount++;
            failedNames.push(tc.name);
            console.error("Import error:", err);
          }
        }

        importProgress.status = "Refreshing data...";
        await loadData();
        showImportModal = false;
        importSummary = {
          success: successCount,
          failed: failedCount,
          failedNames,
          skipped,
        };
      } else {
        const detectedHeaders = headers.join(", ");
        ui.showMessage(
          `No test cases found. Detected headers: ${detectedHeaders}`,
          "error",
        );
      }
    };

    reader.readAsText(file);
    input.value = ""; // Reset
  }

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
  let assignees: any[] = [];

  const fieldOptions = [
    { value: "all", label: "By all fields" },
    { value: "test_no", label: "ID" },
    { value: "name", label: "Title" },
    { value: "suite", label: "Suite" },
    { value: "description", label: "Description" },
    { value: "preconditions", label: "Pre-conditions" },
    { value: "postconditions", label: "Post-conditions" },
    { value: "custom_fields", label: "Custom Fields" },
    { value: "comments", label: "Comments" },
    { value: "steps", label: "Steps" },
  ];

  const priorityOptions = [
    {
      value: "high",
      label: "High",
      icon: SignalHigh,
      iconClass: "text-white",
      pillClass: "bg-orange-600 text-white px-1.5 py-0.5 rounded shadow-sm border border-orange-700",
    },
    {
      value: "medium",
      label: "Medium",
      icon: SignalMedium,
      iconClass: "text-orange-600 dark:text-orange-400",
      pillClass: "bg-orange-900/40 text-orange-600 dark:text-orange-400 px-1.5 py-0.5 rounded border border-orange-900/50",
    },
    {
      value: "low",
      label: "Low",
      icon: SignalLow,
      iconClass: "text-gray-500 dark:text-gray-400",
      pillClass: "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400 px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700",
    },
  ];

  const priorityTone = {
    high: {
      button:
        "bg-orange-600 text-white border-orange-700 shadow-sm",
      menu: "text-orange-600 dark:text-orange-400",
      dot: "bg-orange-600",
      icon: SignalHigh,
    },
    medium: {
      button:
        "bg-orange-900/40 text-orange-600 dark:text-orange-400 border-orange-900/50",
      menu: "text-orange-600 dark:text-orange-400",
      dot: "bg-orange-400",
      icon: SignalMedium,
    },
    low: {
      button:
        "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400 border-gray-200 dark:border-gray-700",
      menu: "text-gray-500 dark:text-gray-400",
      dot: "bg-gray-400",
      icon: SignalLow,
    },
  } satisfies Record<
    TestCase["priority"],
    { button: string; menu: string; dot: string; icon: typeof SignalHigh }
  >;

  function getPriorityMeta(priority: string | null | undefined) {
    return (
      priorityTone[(priority || "medium") as TestCase["priority"]] ||
      priorityTone.medium
    );
  }

  function getPriorityLabel(priority: string | null | undefined) {
    return priorityOptions.find((o) => o.value === priority)?.label || "Medium";
  }

  function patchTestCase(caseId: string, patch: Partial<TestCase>) {
    suites = suites.map((s) => ({
      ...s,
      cases: s.cases.map((c) => (c.id === caseId ? { ...c, ...patch } : c)),
    }));
  }

  const statusOptions = [
    {
      value: "draft",
      label: "Draft",
      icon: CircleDashed,
      iconClass: "text-slate-400",
      pillClass: "text-slate-400 dark:text-slate-300 font-medium",
    },
    {
      value: "actual",
      label: "Actual",
      icon: ListChecks,
      iconClass: "text-blue-400",
      pillClass: "text-blue-500 dark:text-blue-400 font-medium",
    },
    {
      value: "passed",
      label: "Passed",
      icon: CheckCircle2,
      iconClass: "text-emerald-400",
      pillClass: "text-emerald-500 dark:text-emerald-400 font-medium",
    },
    {
      value: "failed",
      label: "Failed",
      icon: XCircle,
      iconClass: "text-red-400",
      pillClass: "text-red-500 dark:text-red-400 font-medium",
    },
    {
      value: "blocked",
      label: "Blocked",
      icon: Ban,
      iconClass: "text-orange-400",
      pillClass: "text-orange-500 dark:text-orange-400 font-medium",
    },
    {
      value: "deprecated",
      label: "Deprecated",
      icon: History,
      iconClass: "text-slate-500",
      pillClass: "text-slate-400 dark:text-slate-500 font-medium",
    },
  ];

  const stepFormatOptions = [
    { value: "gherkin", label: "Gherkin" },
    { value: "classic", label: "Classic" },
  ];

  const fixedOptions = [
    {
      value: "no",
      label: "Not fixed",
      icon: AlertCircle,
      iconClass: "text-amber-400",
    },
    { value: "yes", label: "Fixed", icon: Check, iconClass: "text-green-400" },
  ];

  $: suiteOptions = suites.map((suite) => ({
    value: suite.id,
    label: suite.title,
  }));
  $: assigneeOptions = [
    {
      value: "unassigned",
      label: "Unassigned",
      user_id: null,
      avatarColor: "#64748b",
    },
    ...assignees.map((assignee) => ({
      value: String(assignee.id),
      label: assignee.name,
      user_id: assignee.user_id,
      avatarUrl: assignee.avatar_url || assignee.avatarUrl,
      avatarColor: assignee.color || "#6366f1",
    })),
  ];

  $: availableAssigneeOptions = isAuthorized
    ? assigneeOptions
    : assigneeOptions
        .filter(
          (o) =>
            o.value === "unassigned" || String(o.user_id) === String($user?.id),
        )
        .map((o) =>
          String(o.user_id) === String($user?.id) ? { ...o, label: "me" } : o,
        );

  $: testRunAssigneeOptions = (() => {
    const meEntry = assignees.find(
      (a) => String(a.user_id) === String($user?.id),
    );
    const rest = assigneeOptions.filter(
      (o) =>
        o.value !== "unassigned" && String(o.user_id) !== String($user?.id),
    );
    return [
      {
        value: "unassigned",
        label: "Unassigned",
        user_id: null,
        avatarColor: "#64748b",
      },
      ...(meEntry
        ? [
            {
              value: String(meEntry.id),
              label: "Me",
              user_id: meEntry.user_id,
              avatarUrl: meEntry.avatar_url || meEntry.avatarUrl,
              avatarColor: meEntry.color || "#6366f1",
            },
          ]
        : []),
      ...rest,
    ];
  })();

  $: workspaceId = $page.params.workspace_id;
  $: workspaceLabel = $currentWorkspaceName || "Current workspace";
  $: suiteCount = suites.filter((s) => s.id !== "unassigned").length;
  $: caseCount = suites.reduce(
    (total, suite) => total + (suite.totalCount ?? suite.cases?.length ?? 0),
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

    if (browser) {
      setTimeout(() => {
        const element = document.getElementById(`suite-${suite.id}`);
        if (element) {
          element.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      }, 0);
    }
  }

  function selectCase(suite: Suite, testCase: TestCase) {
    selectedSuiteId = suite.id;
    selectedCaseId = testCase.id;
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
        broadcastTestCaseAssignment({
          id: newCase.id,
          title: newCase.title,
          workspaceId,
          suiteId: newCase.suite_id,
          assignees,
          nextAssigneeIds: [
            newCase.assignee || "unassigned",
            newCase.assign_dev || "unassigned",
          ],
          action: "create",
        });

        suites = suites.map((suite) =>
          suite.id === newCase.suite_id
            ? {
                ...suite,
                cases: [...(suite.cases || []), newCase],
                totalCount: (suite.totalCount ?? suite.cases.length) + 1,
              }
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
            ? {
                ...suite,
                cases: [...(suite.cases || []), newCase],
                totalCount: (suite.totalCount ?? suite.cases.length) + 1,
              }
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

  function clampSidebarWidth(w: number) {
    return Math.min(Math.max(w, 180), 480);
  }

  function startSidebarResize(event: PointerEvent) {
    if (!browser) return;
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = leftSidebarWidth;

    const onMove = (e: PointerEvent) => {
      leftSidebarWidth = clampSidebarWidth(startWidth + e.clientX - startX);
    };
    const onUp = () => {
      localStorage.setItem(
        "test-case-left-sidebar-width",
        String(leftSidebarWidth),
      );
      stopSidebarResize?.();
      stopSidebarResize = null;
    };

    stopSidebarResize?.();
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp, { once: true });
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    stopSidebarResize = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
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
    const savedSidebarWidth = Number(
      localStorage.getItem("test-case-left-sidebar-width"),
    );
    if (Number.isFinite(savedSidebarWidth) && savedSidebarWidth > 0) {
      leftSidebarWidth = clampSidebarWidth(savedSidebarWidth);
    }
    document.addEventListener("click", handleGlobalClick, true);
  });

  onDestroy(() => {
    stopPanelResize?.();
    stopSidebarResize?.();
    document.removeEventListener("click", handleGlobalClick, true);
  });
  function autofocus(node: HTMLElement) {
    node.focus();
  }

  function clickOutside(node: HTMLElement, callback: () => void) {
    const handler = (e: MouseEvent) => {
      if (!node.contains(e.target as Node)) callback();
    };
    document.addEventListener("click", handler, true);
    return {
      destroy: () => document.removeEventListener("click", handler, true),
    };
  }

  function closeContextMenus() {
    suiteMenuId = null;
    runMenuId = null;
  }

  function handleGlobalClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest("[data-context-menu]")) {
      closeContextMenus();
    }
  }
</script>

<svelte:head>
  <title>{$_("meta__testcases_title", { values: { name: workspaceLabel } })}</title>
  <meta name="description" content={$_("meta__testcases_desc", { values: { name: workspaceLabel } })} />
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
        <div class="flex flex-col gap-4">
          <div class="flex items-baseline gap-3">
            <h1
              class="text-2xl font-black tracking-tight text-slate-800 dark:text-white"
            >
              {workspaceLabel} repository
            </h1>
            <span class="text-sm font-medium text-slate-500">
              {caseCount} cases | {suiteCount} suites
            </span>
          </div>

          <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="flex flex-wrap items-center gap-3">
              <div
                class="group flex h-10 min-w-[320px] rounded-xl border border-slate-200 bg-white shadow-sm transition-all focus-within:border-indigo-500 focus-within:ring-4 focus-within:ring-indigo-500/10 dark:border-gray-700 dark:bg-gray-900 dark:focus-within:border-indigo-400"
              >
                <label class="flex min-w-0 flex-1 items-center gap-2.5 px-3.5">
                  <Search
                    size={18}
                    class="shrink-0 text-slate-400 group-focus-within:text-indigo-500"
                  />
                  <input
                    bind:value={query}
                    class="min-w-0 flex-1 border-0 bg-transparent text-sm font-bold text-slate-700 outline-none placeholder:text-slate-400 dark:!bg-transparent dark:text-gray-200"
                    placeholder={$_("tc__ph_search")}
                  />
                  {#if query}
                    <button
                      on:click={() => (query = "")}
                      class="p-1 hover:bg-slate-100 rounded-full dark:hover:bg-gray-800"
                    >
                      <X size={14} class="text-slate-400" />
                    </button>
                  {/if}
                </label>
                <div
                  class="flex h-full items-center w-48 rounded-r-xl border-l border-slate-200 bg-slate-50/50 dark:border-gray-700 dark:bg-gray-800/50"
                >
                  <SearchableSelect
                    id="test-case-search-field"
                    bind:value={field}
                    options={fieldOptions}
                    showSearch={false}
                    minimal={true}
                  />
                </div>
              </div>

              <!-- Active Filter Chips -->
              {#each Object.entries(activeFilters) as [prop, values]}
                {#if values.length > 0}
                  <div
                    class="flex items-center gap-1.5 rounded-lg bg-indigo-50 border border-indigo-100 px-2.5 py-1.5 text-[13px] font-black text-indigo-700 dark:bg-indigo-500/10 dark:border-indigo-500/20 dark:text-indigo-300 shadow-sm"
                  >
                    <span class="opacity-60"
                      >{filterProperties.find((p) => p.id === prop)?.label ||
                        prop}:</span
                    >
                    <span
                      >{values
                        .map((v) => getFilterLabel(prop, v))
                        .join(", ")}</span
                    >
                    <button
                      class="ml-1 rounded-full p-0.5 hover:bg-indigo-200 dark:hover:bg-indigo-500/30 transition-colors"
                      on:click={() => {
                        activeFilters[prop] = [];
                        loadData(query, field);
                      }}
                    >
                      <X size={12} strokeWidth={3} />
                    </button>
                  </div>
                {/if}
              {/each}

              <!-- Add Filter Dropdown -->
              <div class="relative">
                <button
                  class="flex h-10 items-center gap-2 rounded-xl bg-slate-100 px-4 text-[13px] font-black text-slate-600 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700 transition-all active:scale-95"
                  on:click={() => {
                    showFilterDropdown = !showFilterDropdown;
                    activeFilterProperty = null;
                  }}
                >
                  <Plus size={16} />
                  Add filter
                </button>

                {#if showFilterDropdown}
                  <!-- svelte-ignore a11y_consider_explicit_label -->
                  <button
                    class="fixed inset-0 z-40 h-full w-full cursor-default border-none bg-transparent"
                    on:click={() => (showFilterDropdown = false)}
                  ></button>

                  <div
                    class="absolute left-0 top-full z-50 mt-1 w-64 overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xl dark:border-gray-700 dark:bg-gray-800 animate-in fade-in slide-in-from-top-1 duration-200"
                  >
                    {#if !activeFilterProperty}
                      <div class="flex flex-col py-1">
                        <div
                          class="px-4 py-2 text-[10px] font-black uppercase tracking-wider text-slate-400"
                        >
                          Filter by
                        </div>
                        {#each filterProperties as prop}
                          <button
                            class="flex w-full items-center justify-between px-4 py-2.5 text-left text-sm font-bold text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                            on:click={() => {
                              activeFilterProperty = prop.id;
                            }}
                          >
                            <span>{prop.label}</span>
                            <ChevronRight size={14} class="opacity-30" />
                          </button>
                        {/each}
                        <div
                          class="h-px bg-slate-100 dark:bg-gray-700 my-1"
                        ></div>
                        <button
                          class="px-4 py-2 text-sm font-black text-indigo-600 hover:bg-indigo-50 dark:text-indigo-400 dark:hover:bg-indigo-500/10 text-center w-full transition-colors"
                          on:click={() => {
                            activeFilters = {
                              priority: [],
                              status: [],
                              fixed: [],
                              assign_dev: [],
                            };
                            loadData(query, field);
                            showFilterDropdown = false;
                          }}
                        >
                          Clear selection
                        </button>
                      </div>
                    {:else}
                      <div
                        class="flex flex-col py-1 max-h-[400px] overflow-y-auto"
                      >
                        <div class="flex items-center gap-2 px-3 py-2">
                          <button
                            class="p-1 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg dark:hover:bg-gray-700 dark:hover:text-gray-200 transition-colors"
                            on:click={() => (activeFilterProperty = null)}
                          >
                            <ChevronLeft size={16} />
                          </button>
                          <span
                            class="text-[10px] font-black uppercase tracking-wider text-slate-400"
                            >{filterProperties.find(
                              (p) => p.id === activeFilterProperty,
                            )?.label}</span
                          >
                        </div>
                        <div
                          class="h-px bg-slate-100 dark:bg-gray-700 mb-1"
                        ></div>

                        <!-- Render options based on property -->
                        {#if activeFilterProperty === "priority"}
                          {#each priorityOptions as opt}
                            <label
                              class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                            >
                              <input
                                type="checkbox"
                                class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-600 w-4 h-4"
                                checked={activeFilters.priority.includes(
                                  opt.value,
                                )}
                                on:change={() =>
                                  toggleFilterValue("priority", opt.value)}
                              />
                              <span
                                class="text-sm font-bold text-slate-700 dark:text-gray-200"
                                >{opt.label}</span
                              >
                            </label>
                          {/each}
                        {:else if activeFilterProperty === "status"}
                          {#each statusOptions as opt}
                            <label
                              class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                            >
                              <input
                                type="checkbox"
                                class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-600 w-4 h-4"
                                checked={activeFilters.status.includes(
                                  opt.value,
                                )}
                                on:change={() =>
                                  toggleFilterValue("status", opt.value)}
                              />
                              <span
                                class="text-sm font-bold text-slate-700 dark:text-gray-200"
                                >{opt.label}</span
                              >
                            </label>
                          {/each}
                        {:else if activeFilterProperty === "fixed"}
                          {#each fixedOptions as opt}
                            <label
                              class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                            >
                              <input
                                type="checkbox"
                                class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-600 w-4 h-4"
                                checked={activeFilters.fixed.includes(
                                  opt.value,
                                )}
                                on:change={() =>
                                  toggleFilterValue("fixed", opt.value)}
                              />
                              <span
                                class="text-sm font-bold text-slate-700 dark:text-gray-200"
                                >{opt.label}</span
                              >
                            </label>
                          {/each}
                        {:else if activeFilterProperty === "assign_dev"}
                          {#each availableAssigneeOptions as opt}
                            <label
                              class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                            >
                              <input
                                type="checkbox"
                                class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-600 w-4 h-4"
                                checked={activeFilters.assign_dev.includes(
                                  opt.value,
                                )}
                                on:change={() =>
                                  toggleFilterValue("assign_dev", opt.value)}
                              />
                              <span
                                class="text-sm font-bold text-slate-700 dark:text-gray-200"
                                >{opt.label}</span
                              >
                            </label>
                          {/each}
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="h-6 w-px bg-slate-200 dark:bg-gray-800"></div>
              <div class="relative flex items-center gap-2">
                <button
                  class="flex h-9 w-9 items-center justify-center rounded-lg text-slate-600 hover:bg-slate-100 dark:text-gray-300 dark:hover:bg-gray-800 transition-colors"
                  on:click={() => (showActionsMenu = !showActionsMenu)}
                  title={$_("tc__title_actions")}
                >
                  <MoreHorizontal size={16} />
                </button>

                {#if showActionsMenu}
                  <!-- svelte-ignore a11y_consider_explicit_label -->
                  <button
                    class="fixed inset-0 z-40 h-full w-full cursor-default border-none bg-transparent"
                    on:click={() => (showActionsMenu = false)}
                  ></button>

                  <div
                    class="absolute right-0 top-full z-50 mt-1 w-56 overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xl dark:border-gray-700 dark:bg-gray-800 animate-in fade-in slide-in-from-top-1 duration-200"
                  >
                    <div class="flex flex-col py-1">
                      <button
                        class="flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm font-black text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                        on:click={() => {
                          downloadTemplate();
                          showActionsMenu = false;
                        }}
                      >
                        <Download size={16} class="text-slate-400" />
                        {$_("testCases__download_template")}
                      </button>

                      {#if isAuthorized}
                        <label
                          class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-left text-sm font-black text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                        >
                          <FileUp size={16} class="text-slate-400" />
                          {$_("testCases__import")}
                          <input
                            type="file"
                            accept=".csv"
                            class="hidden"
                            on:change={(e) => {
                              handleImport(e);
                              showActionsMenu = false;
                            }}
                          />
                        </label>
                      {/if}

                      <button
                        class="flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm font-black text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                        on:click={() => {
                          exportToCSV();
                          showActionsMenu = false;
                        }}
                      >
                        <FileDown size={16} class="text-slate-400" />
                        {$_("testCases__export")}
                      </button>

                      <div
                        class="my-1 border-t border-slate-100 dark:border-gray-700"
                      ></div>

                      <button
                        class="flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm font-black text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                        on:click={() => {
                          window.open(
                            `${base}/public/${workspaceId}/test-cases/`,
                            "_blank",
                          );
                          showActionsMenu = false;
                        }}
                      >
                        <ExternalLink size={16} class="text-slate-400" />
                        {$_("testCases__public_view")}
                      </button>
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>
      </header>

      <div class="flex min-h-0 flex-1">
        <aside
          class="relative hidden shrink-0 border-r border-gray-200 bg-white lg:block dark:border-gray-800 dark:bg-gray-950"
          style="width: {leftSidebarWidth}px"
        >
          <button
            class="absolute right-0 top-0 z-20 h-full w-1.5 cursor-col-resize border-0 bg-transparent transition-colors hover:bg-indigo-500/20 focus:bg-indigo-500/20 focus:outline-none"
            aria-label="Resize sidebar"
            on:pointerdown={startSidebarResize}
          ></button>
          <div class="h-full overflow-y-auto pl-4 pr-3 pt-4 pb-4">
            <div class="mb-2 flex items-center gap-2">
              <button
                class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors"
                title={$_("tc__title_toggle_suites")}
                on:click={() => (isSuitesOpen = !isSuitesOpen)}
              >
                <ChevronDown
                  size={16}
                  class="transition-transform duration-200 {isSuitesOpen
                    ? ''
                    : '-rotate-90'}"
                />
              </button>
              <h2
                class="text-lg font-black text-slate-800 dark:text-white flex-1"
              >
                Suites
              </h2>
              {#if isAuthorized}
                <button
                  class="grid h-8 w-8 place-items-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
                  title={$_("tc__title_add_suite")}
                  on:click={() => (showCreateSuiteModal = true)}
                >
                  <Plus size={16} />
                </button>
              {/if}
            </div>

            {#if isSuitesOpen}
              <div class="space-y-1">
                {#each suites as suite}
                  <div
                    class="group relative flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-sm transition-colors {selectedSuiteId ===
                    suite.id
                      ? 'bg-slate-100 dark:bg-gray-800'
                      : 'hover:bg-slate-50 dark:hover:bg-gray-900'}"
                  >
                    <button
                      class="flex flex-1 min-w-0 items-center gap-2 text-left {selectedSuiteId ===
                      suite.id
                        ? 'font-black text-slate-800 dark:text-white'
                        : 'font-bold text-slate-600 dark:text-gray-400'}"
                      on:click={() => selectSuite(suite)}
                    >
                      <ChevronDown size={14} class="shrink-0 text-slate-500" />
                      <span class="min-w-0 flex-1 truncate">{suite.title}</span>
                      <span class="text-xs font-black text-slate-500"
                        >{suite.totalCount ?? suite.cases?.length ?? 0}</span
                      >
                    </button>
                    {#if isAuthorized}
                      <div class="relative shrink-0" data-context-menu>
                        <button
                          class="grid h-6 w-6 place-items-center rounded text-slate-400 opacity-0 group-hover:opacity-100 hover:bg-slate-200 dark:hover:bg-gray-700 transition-all"
                          on:click|stopPropagation={(e) =>
                            openContextMenu(e, "suite", suite.id)}
                        >
                          <MoreHorizontal size={14} />
                        </button>
                        {#if suiteMenuId === suite.id}
                          <div
                            class="fixed z-[9999] min-w-[140px] rounded-xl border border-gray-200 dark:border-white/10 bg-white dark:bg-slate-900 shadow-xl py-1"
                            style="top: {contextMenuY}px; left: {contextMenuX}px"
                            data-context-menu
                          >
                            <button
                              class="flex w-full items-center gap-2.5 px-3 py-2 text-[13px] font-bold text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
                              on:click|stopPropagation={() => {
                                suiteToDelete = suite;
                                showDeleteSuiteModal = true;
                                suiteMenuId = null;
                              }}
                            >
                              <Trash2 size={14} />
                              Delete suite
                            </button>
                          </div>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Test Runs Section -->
            <div
              class="mt-6 border-t border-gray-100 dark:border-gray-800 pt-4"
            >
              <div class="mb-2 flex items-center gap-2">
                <button
                  class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors"
                  title={$_("tc__title_toggle_runs")}
                  on:click={() => (isTestRunsOpen = !isTestRunsOpen)}
                >
                  <ChevronDown
                    size={16}
                    class="transition-transform duration-200 {isTestRunsOpen
                      ? ''
                      : '-rotate-90'}"
                  />
                </button>
                <h2
                  class="text-lg font-black text-slate-800 dark:text-white flex-1"
                >
                  Test Runs
                </h2>
                <span class="text-xs font-black text-slate-400"
                  >{testRunsTotal || testRuns.length}</span
                >
                {#if isAuthorized}
                  <button
                    class="grid h-8 w-8 place-items-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
                    title={$_("tc__title_new_run")}
                    on:click={openNewTestRunModal}
                  >
                    <Plus size={16} />
                  </button>
                {/if}
              </div>

              {#if isTestRunsOpen}
                {#if testRuns.length === 0}
                  <p
                    class="px-2 py-2 text-xs text-slate-400 dark:text-gray-500"
                  >
                    No test runs yet
                  </p>
                {:else}
                  <div class="space-y-1">
                    {#each testRuns as run}
                      <div
                        class="group relative flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-sm font-bold transition-colors {selectedRunDetail?.id ===
                        run._id
                          ? 'bg-indigo-50 dark:bg-indigo-500/10'
                          : 'hover:bg-slate-50 dark:hover:bg-gray-900'}"
                      >
                        <button
                          class="flex flex-1 min-w-0 items-center gap-2 text-left {selectedRunDetail?.id ===
                          run._id
                            ? 'text-indigo-700 dark:text-indigo-400'
                            : 'text-slate-600 dark:text-gray-400'}"
                          on:click={() => openTestRun(run)}
                        >
                          <div
                            class="h-2 w-2 shrink-0 rounded-full {run.status ===
                            'completed'
                              ? 'bg-emerald-500'
                              : run.status === 'running'
                                ? 'bg-amber-400'
                                : 'bg-slate-300 dark:bg-gray-600'}"
                          ></div>
                          <span class="min-w-0 flex-1 truncate">{run.name}</span
                          >
                          <span class="text-xs font-black text-slate-400"
                            >{run.test_cases.length}</span
                          >
                        </button>
                        {#if isAuthorized}
                          <div class="relative shrink-0" data-context-menu>
                            <button
                              class="grid h-6 w-6 place-items-center rounded text-slate-400 opacity-0 group-hover:opacity-100 hover:bg-slate-200 dark:hover:bg-gray-700 transition-all"
                              on:click|stopPropagation={(e) =>
                                openContextMenu(e, "run", run._id)}
                            >
                              <MoreHorizontal size={14} />
                            </button>
                            {#if runMenuId === run._id}
                              <div
                                class="fixed z-[9999] min-w-[140px] rounded-xl border border-gray-200 dark:border-white/10 bg-white dark:bg-slate-900 shadow-xl py-1"
                                style="top: {contextMenuY}px; left: {contextMenuX}px"
                                data-context-menu
                              >
                                <button
                                  class="flex w-full items-center gap-2.5 px-3 py-2 text-[13px] font-bold text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
                                  on:click|stopPropagation={() => {
                                    runToDelete = run;
                                    showDeleteRunModal = true;
                                    runMenuId = null;
                                  }}
                                >
                                  <Trash2 size={14} />
                                  Delete run
                                </button>
                              </div>
                            {/if}
                          </div>
                        {/if}
                      </div>
                    {/each}
                    {#if testRuns.length < testRunsTotal}
                      <button
                        class="flex w-full items-center justify-center gap-1.5 mt-2 px-3 py-1.5 rounded-lg text-xs font-bold text-slate-500 dark:text-gray-400 bg-slate-100 dark:bg-gray-800 hover:bg-indigo-50 hover:text-indigo-600 dark:hover:bg-indigo-500/10 dark:hover:text-indigo-400 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                        disabled={isLoadingMoreRuns}
                        on:click={loadMoreTestRuns}
                      >
                        {#if isLoadingMoreRuns}
                          <svg
                            class="animate-spin w-3 h-3"
                            fill="none"
                            viewBox="0 0 24 24"
                          >
                            <circle
                              class="opacity-25"
                              cx="12"
                              cy="12"
                              r="10"
                              stroke="currentColor"
                              stroke-width="4"
                            ></circle>
                            <path
                              class="opacity-75"
                              fill="currentColor"
                              d="M4 12a8 8 0 018-8v8z"
                            ></path>
                          </svg>
                          <span>{$_("tc__loading")}</span>
                        {:else}
                          <svg
                            class="w-3 h-3"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              stroke-width="2.5"
                              d="M19 9l-7 7-7-7"
                            />
                          </svg>
                          <span>{testRunsTotal - testRuns.length} more</span>
                        {/if}
                      </button>
                    {/if}
                  </div>
                {/if}
              {/if}
            </div>
          </div>
        </aside>

        <main class="min-w-0 flex-1 overflow-y-auto bg-white dark:bg-gray-950">
          {#if viewMode === "test-run"}
            <!-- ══ TEST RUN VIEW ══ -->
            <div class="flex flex-col h-full">
              <!-- Run header -->
              <div
                class="flex items-center gap-3 px-6 py-4 border-b border-slate-100 dark:border-gray-800 shrink-0"
              >
                <button
                  class="grid h-8 w-8 place-items-center rounded-lg text-slate-400 hover:bg-slate-100 dark:hover:bg-gray-800"
                  on:click={closeTestRun}
                >
                  <ChevronLeft size={18} />
                </button>
                <div class="flex items-center gap-3 min-w-0 flex-1">
                  <h2
                    class="text-lg font-black text-slate-800 dark:text-white truncate"
                  >
                    {selectedRunDetail?.name ?? "Test run"}
                  </h2>
                  {#if selectedRunDetail}
                    <span
                      class="shrink-0 rounded-full px-2.5 py-0.5 text-xs font-black
                      {selectedRunDetail.status === 'completed'
                        ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400'
                        : selectedRunDetail.status === 'aborted'
                          ? 'bg-red-100 text-red-700 dark:bg-red-500/20 dark:text-red-400'
                          : 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400'}"
                    >
                      {selectedRunDetail.status === "completed"
                        ? "Completed"
                        : selectedRunDetail.status === "aborted"
                          ? "Aborted"
                          : "Running"}
                    </span>
                  {/if}
                </div>
                {#if isAuthorized}
                  {#if selectedRunDetail?.status === "running"}
                    <button
                      class="shrink-0 rounded-xl bg-emerald-600 px-4 py-1.5 text-xs font-black text-white hover:bg-emerald-500"
                      on:click={() => updateRunStatus("completed")}
                      >{$_("tc__mark_complete")}</button
                    >
                  {:else if selectedRunDetail?.status === "completed"}
                    <button
                      class="shrink-0 rounded-xl bg-amber-500 px-4 py-1.5 text-xs font-black text-white hover:bg-amber-400"
                      on:click={() => updateRunStatus("running")}>{$_("tc__reopen")}</button
                    >
                  {/if}
                  <button
                    class="shrink-0 grid h-8 w-8 place-items-center rounded-lg text-slate-400 hover:bg-red-50 hover:text-red-500 dark:hover:bg-red-500/10 dark:hover:text-red-400 transition-colors"
                    title={$_("tc__title_delete_run")}
                    on:click={() => (showDeleteRunModal = true)}
                  >
                    <Trash2 size={16} />
                  </button>
                {/if}
              </div>

              <!-- Stats bar -->
              {#if selectedRunDetail}
                {@const s = selectedRunDetail.stats}
                <div
                  class="flex items-center gap-5 px-6 py-3 shrink-0 border-b border-slate-100 dark:border-gray-800 text-xs font-bold flex-wrap"
                >
                  <span class="text-slate-400">{s.total} total</span>
                  {#if s.passed}
                    <span class="text-emerald-600 dark:text-emerald-400"
                      >{s.passed} passed</span
                    >
                  {/if}
                  {#if s.failed}
                    <span class="text-red-500">{s.failed} failed</span>
                  {/if}
                  {#if s.blocked}
                    <span class="text-orange-500">{s.blocked} blocked</span>
                  {/if}
                  {#if s.skipped}
                    <span class="text-blue-400">{s.skipped} skipped</span>
                  {/if}
                  {#if s.invalid}
                    <span class="text-purple-500">{s.invalid} invalid</span>
                  {/if}
                  {#if s.pending}
                    <span class="text-slate-400">{s.pending} pending</span>
                  {/if}
                  {#if selectedRunDetail.created_by_name || selectedRunDetail.created_by}
                    <span class="flex items-center gap-1.5 text-slate-400">
                      Created by
                      <span class="flex items-center gap-1">
                        <span
                          class="shrink-0 h-4 w-4 rounded-full overflow-hidden flex items-center justify-center text-[9px] font-bold text-white"
                          style="background-color: {selectedRunDetail.created_by_color || '#6366f1'}"
                        >
                          {#if selectedRunDetail.created_by_avatar_url}
                            <img src={selectedRunDetail.created_by_avatar_url} alt="" class="w-full h-full object-cover" />
                          {:else}
                            {(selectedRunDetail.created_by_name || selectedRunDetail.created_by || '?')[0].toUpperCase()}
                          {/if}
                        </span>
                        <span class="text-slate-600 dark:text-gray-300 font-bold">{selectedRunDetail.created_by_name || selectedRunDetail.created_by}</span>
                      </span>
                    </span>
                  {/if}
                  <div
                    class="flex-1 min-w-[80px] h-1.5 rounded-full bg-slate-100 dark:bg-gray-800 overflow-hidden"
                  >
                    <div
                      class="h-full bg-emerald-500 rounded-full transition-all"
                      style="width:{s.total
                        ? Math.round((s.passed / s.total) * 100)
                        : 0}%"
                    ></div>
                  </div>
                </div>
              {/if}

              <!-- Test case list -->
              <div class="flex-1 overflow-y-auto">
                {#if isLoadingRunDetail}
                  <div
                    class="flex items-center justify-center h-48 text-slate-400 dark:text-gray-500"
                  >
                    <div
                      class="h-6 w-6 animate-spin rounded-full border-2 border-slate-300 border-t-indigo-500"
                    ></div>
                  </div>
                {:else if selectedRunDetail}
                  <!-- Header row -->
                  <div
                    class="grid grid-cols-[60px_52px_1fr_140px] items-center gap-3 px-5 py-2 border-b border-slate-100 dark:border-gray-800 bg-slate-50 dark:bg-gray-900/60 text-xs font-black uppercase tracking-widest text-slate-400"
                  >
                    <span>#</span>
                    <span></span>
                    <span>{$_("tc__test_case")}</span>
                    <span>{$_("tc__status")}</span>
                  </div>
                  <div class="divide-y divide-slate-100 dark:divide-gray-800">
                    {#each selectedRunDetail.test_cases as entry}
                      <div
                        class="grid grid-cols-[60px_52px_1fr_140px] items-center gap-3 pl-4 pr-5 py-3 transition-colors border-l-4
                        {entry.status === 'passed'
                          ? 'border-l-emerald-500 bg-emerald-500/5 hover:bg-emerald-500/10'
                          : entry.status === 'failed'
                            ? 'border-l-red-500    bg-red-500/5    hover:bg-red-500/10'
                            : entry.status === 'blocked'
                              ? 'border-l-orange-500 bg-orange-500/5 hover:bg-orange-500/10'
                              : entry.status === 'skipped'
                                ? 'border-l-blue-400   bg-blue-400/5   hover:bg-blue-400/10'
                                : entry.status === 'invalid'
                                  ? 'border-l-purple-500 bg-purple-500/5 hover:bg-purple-500/10'
                                  : 'border-l-transparent hover:bg-slate-50 dark:hover:bg-gray-900/40'}"
                      >
                        <!-- TC-no -->
                        <button
                          class="font-mono text-sm font-bold text-slate-400 hover:text-indigo-500 dark:hover:text-indigo-400 transition-colors text-left"
                          title={$_("tc__title_view_case")}
                          on:click={() => {
                            if (!entry.test_case) return;
                            selectedSuiteId =
                              entry.test_case.suite_id || suites[0]?.id || "";
                            selectedCaseId = entry.test_case.id;
                            showDetail = true;
                          }}>TC-{entry.test_case?.test_no ?? "?"}</button
                        >
                        <!-- Priority badge -->
                        <span
                          class="flex items-center justify-center gap-1 rounded border px-1.5 py-0.5 text-[10px] font-medium shadow-sm
                          {entry.test_case?.priority === 'high'
                            ? 'bg-orange-600 text-white border-orange-700'
                            : entry.test_case?.priority === 'medium'
                              ? 'bg-orange-900/40 text-orange-600 dark:text-orange-400 border-orange-900/50'
                              : 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400 border-gray-200 dark:border-gray-700'}"
                        >
                          {entry.test_case?.priority ?? "—"}
                        </span>
                        <!-- Name -->
                        <span
                          class="min-w-0 truncate text-sm font-bold text-slate-700 dark:text-gray-200"
                        >
                          {entry.test_case?.name ?? entry.test_case_id}
                        </span>
                        <!-- Status -->
                        {#if isAuthorized}
                          <div
                            class="property-select w-[130px]"
                            class:pointer-events-none={updatingCaseId ===
                              entry.test_case_id ||
                              selectedRunDetail?.status === "completed"}
                            class:opacity-50={updatingCaseId ===
                              entry.test_case_id ||
                              selectedRunDetail?.status === "completed"}
                          >
                            <SearchableSelect
                              id="run-case-status-{entry.test_case_id}"
                              value={entry.status}
                              options={RUN_CASE_STATUSES}
                              showSearch={false}
                              minimal={true}
                              dropdownClass="w-full"
                              on:select={(e) =>
                                updateRunCaseStatus(
                                  entry.test_case_id,
                                  e.detail,
                                )}
                            />
                          </div>
                        {:else}
                          <div
                            class="property-select w-[130px] pointer-events-none opacity-70"
                          >
                            <SearchableSelect
                              id="run-case-status-ro-{entry.test_case_id}"
                              value={entry.status}
                              options={RUN_CASE_STATUSES}
                              showSearch={false}
                              minimal={true}
                              dropdownClass="w-full"
                              on:select={() => {}}
                            />
                          </div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <!-- ══ SUITES VIEW ══ -->
            <div class="space-y-7 p-4">
              {#each filteredSuites as suite}
                <section id="suite-{suite.id}" class="scroll-mt-4">
                  <div
                    class="flex min-h-12 items-center gap-3 rounded-t-lg px-5 transition-colors duration-300 {selectedSuiteId ===
                    suite.id
                      ? 'bg-indigo-50 dark:bg-indigo-500/10 border-b border-indigo-100 dark:border-indigo-500/20'
                      : 'bg-slate-100 dark:bg-gray-900'}"
                  >
                    <h3
                      class="min-w-0 flex-1 truncate text-base font-black text-slate-700 dark:text-white"
                    >
                      {suite.title}
                    </h3>
                    {#if isAuthorized}
                      <button
                        class="grid h-8 w-8 place-items-center rounded-md text-indigo-600 hover:bg-white dark:text-indigo-400 dark:hover:bg-gray-800"
                        title={$_("tc__title_add_case")}
                        on:click={() => openTestCaseEditor(suite.id)}
                      >
                        <Plus size={16} />
                      </button>
                      <button
                        class="grid h-8 w-8 place-items-center rounded-md text-slate-400 hover:bg-white hover:text-slate-700 dark:hover:bg-gray-800 dark:hover:text-white"
                        title={$_("tc__title_edit_suite")}
                        on:click={() => (editingSuite = { ...suite })}
                      >
                        <Edit3 size={15} />
                      </button>
                      <button
                        class="grid h-8 w-8 place-items-center rounded-md text-slate-400 hover:bg-white hover:text-rose-600 dark:hover:bg-gray-800 dark:hover:text-rose-400"
                        title={$_("tc__title_delete_suite")}
                        on:click={() => {
                          suiteToDelete = suite;
                          deleteMode =
                            suite.id === "unassigned" ? "delete" : "move";
                          showDeleteSuiteModal = true;
                        }}
                      >
                        <Trash2 size={15} />
                      </button>
                    {/if}
                  </div>

                  <div class="divide-y divide-slate-200 dark:divide-gray-800">
                    {#each suite.cases || [] as testCase}
                      <div
                        class="grid w-full grid-cols-[24px_minmax(0,1fr)_auto] items-center gap-3 rounded-md px-3 py-2 text-left transition-colors sm:grid-cols-[108px_60px_minmax(0,1fr)_auto_auto] {selectedCaseId ===
                        testCase.id
                          ? 'bg-slate-100 dark:bg-transparent'
                          : 'hover:bg-slate-50 dark:hover:bg-gray-900/70'}"
                      >
                        <button
                          type="button"
                          class="grid h-5 w-5 place-items-center sm:hidden"
                          aria-label="Open TC-{testCase.test_no}"
                          on:click={() => {
                            selectCase(suite, testCase);
                            showDetail = true;
                          }}
                        >
                          <span
                            class="h-2.5 w-2.5 rounded-full border-2 {testCase.priority ===
                            'high'
                              ? 'border-orange-600'
                              : testCase.priority === 'medium'
                                ? 'border-orange-400'
                                : 'border-gray-300'}"
                          ></span>
                        </button>
                        <div class="relative hidden sm:block">
                          {#if isAuthorized}
                            <button
                              type="button"
                              class="inline-flex items-center justify-between gap-1 rounded border px-1.5 py-0.5 text-[10px] font-medium shadow-sm transition-colors disabled:cursor-wait disabled:opacity-70 {getPriorityMeta(
                                testCase.priority,
                              ).button}"
                              title="Change priority"
                              aria-label="Change priority for TC-{testCase.test_no}"
                              disabled={updatingPriorityCaseIds.has(testCase.id)}
                              on:click|stopPropagation={() =>
                                (openPriorityMenuId =
                                  openPriorityMenuId === testCase.id
                                    ? ""
                                    : testCase.id)}
                            >
                              <span class="inline-flex min-w-0 items-center gap-1">
                                {#if updatingPriorityCaseIds.has(testCase.id)}
                                  <LoaderCircle size={10} class="shrink-0 animate-spin" />
                                {:else}
                                  <svelte:component
                                    this={getPriorityMeta(testCase.priority).icon}
                                    size={10}
                                    class="shrink-0"
                                  />
                                {/if}
                                <span class="truncate">
                                  {getPriorityLabel(testCase.priority)}
                                </span>
                              </span>
                              <ChevronDown
                                size={10}
                                class="shrink-0 opacity-60 transition-transform {openPriorityMenuId ===
                                testCase.id
                                  ? 'rotate-180'
                                  : ''}"
                              />
                            </button>

                            {#if openPriorityMenuId === testCase.id}
                              <button
                                type="button"
                                class="fixed inset-0 z-[8998] cursor-default bg-transparent"
                                aria-label="Close priority menu"
                                on:click|stopPropagation={() =>
                                  (openPriorityMenuId = "")}
                              ></button>
                              <div
                                class="absolute left-0 top-full z-[8999] mt-1 w-36 overflow-hidden rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-gray-900 p-1 shadow-xl"
                              >
                                {#each priorityOptions as opt}
                                  {@const optionMeta = getPriorityMeta(String(opt.value))}
                                  {@const OptionIcon = optionMeta.icon}
                                  <button
                                    type="button"
                                    class="flex h-8 w-full items-center justify-between rounded-md px-2 text-left text-[11px] font-medium transition-colors hover:bg-gray-100 dark:hover:bg-white/[0.05] {testCase.priority ===
                                    opt.value
                                      ? 'bg-gray-100 dark:bg-white/[0.08]'
                                      : 'text-gray-500 dark:text-slate-400'}"
                                    on:click|stopPropagation={() =>
                                      updatePriority(
                                        testCase,
                                        String(opt.value),
                                      )}
                                  >
                                    <span class="inline-flex items-center gap-2 {optionMeta.menu}">
                                      <span
                                        class="h-1.5 w-1.5 rounded-full {optionMeta.dot}"
                                      ></span>
                                      <svelte:component
                                        this={OptionIcon}
                                        size={11}
                                        class="shrink-0"
                                      />
                                      <span>{opt.label}</span>
                                    </span>
                                    {#if testCase.priority === opt.value}
                                      <Check size={12} class="text-gray-400 dark:text-slate-400" />
                                    {/if}
                                  </button>
                                {/each}
                              </div>
                            {/if}
                          {:else}
                            <span
                              class="inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-[10px] font-medium shadow-sm {getPriorityMeta(
                                testCase.priority,
                              ).button}"
                            >
                              <svelte:component
                                this={getPriorityMeta(testCase.priority).icon}
                                size={10}
                                class="shrink-0"
                              />
                              {getPriorityLabel(testCase.priority)}
                            </span>
                          {/if}
                        </div>
                        <button
                          type="button"
                          class="hidden font-mono text-sm font-bold text-slate-400 text-left sm:block"
                          on:click={() => {
                            selectCase(suite, testCase);
                            showDetail = true;
                          }}
                        >
                          TC-{testCase.test_no}
                        </button>
                        <button
                          type="button"
                          class="min-w-0 truncate text-sm font-bold text-slate-700 dark:text-gray-200 text-left"
                          on:click={() => {
                            selectCase(suite, testCase);
                            showDetail = true;
                          }}
                        >
                          {testCase.title}
                        </button>
                        <div class="flex items-center gap-2">
                          {#if isAuthorized}
                            <div class="property-select min-w-[80px]" data-status={testCase.status}>
                              <SearchableSelect
                                id="status-update-{testCase.id}"
                                value={testCase.status}
                                options={statusOptions}
                                showSearch={false}
                                minimal={true}
                                on:select={(e) =>
                                  updateStatus(testCase, e.detail)}
                              />
                            </div>
                          {:else}
                            <div
                              class="property-select min-w-[80px] pointer-events-none opacity-80"
                              data-status={testCase.status}
                            >
                              <SearchableSelect
                                id="status-update-{testCase.id}"
                                value={testCase.status}
                                options={statusOptions}
                                showSearch={false}
                                minimal={true}
                                on:select={() => {}}
                              />
                            </div>
                          {/if}
                          <div class="property-select min-w-[90px]">
                            <SearchableSelect
                              id="fixed-update-{testCase.id}"
                              value={testCase.fixed}
                              options={fixedOptions}
                              showSearch={false}
                              minimal={true}
                              on:select={(e) => updateFixed(testCase, e.detail)}
                            />
                          </div>
                          <div class="property-select min-w-[120px]">
                            <SearchableSelect
                              id="assigndev-update-{testCase.id}"
                              value={testCase.assign_dev || "unassigned"}
                              options={availableAssigneeOptions}
                              showSearch={true}
                              minimal={true}
                              on:select={(e) =>
                                updateAssignDev(testCase, e.detail)}
                            />
                          </div>
                        </div>
                        <div class="flex items-center gap-1">
                          <button
                            class="grid h-8 w-8 place-items-center rounded-lg text-slate-400 hover:bg-white hover:text-slate-700 dark:hover:bg-gray-800 dark:hover:text-white transition-colors"
                            title={isAuthorized ? $_("tc__title_edit") : $_("tc__title_view_case_auth")}
                            on:click={() =>
                              openTestCaseEditor(suite.id, testCase.id)}
                          >
                            {#if isAuthorized}
                              <Edit3 size={16} />
                            {:else}
                              <Eye size={16} />
                            {/if}
                          </button>

                          {#if isAuthorized}
                            <button
                              class="grid h-8 w-8 place-items-center rounded-lg text-slate-400 hover:bg-white hover:text-slate-700 dark:hover:bg-gray-800 dark:hover:text-white transition-colors"
                              title={$_("tc__title_delete_case")}
                              on:click={() => deleteTestCase(testCase.id)}
                            >
                              <Trash2 size={16} />
                            </button>
                          {/if}
                        </div>
                      </div>
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
                      {#if isAuthorized}
                        {#if quickTestInputSuiteId === suite.id}
                          <input
                            bind:value={quickTestTitle}
                            class="flex-1 border-0 !bg-transparent text-sm font-bold outline-none placeholder:text-slate-400 dark:text-white"
                            placeholder={$_("testCases__title_placeholder")}
                            on:keydown={(e) => {
                              if (e.key === "Enter")
                                handleQuickCreate(suite.id);
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
                      {:else}
                        <div class="flex-1"></div>
                      {/if}

                      <div class="flex gap-2 ml-auto">
                        {#if suite.page > 1 || suite.hasMore}
                          <button
                            class="flex items-center gap-1 rounded-md border border-slate-200 px-3 py-1.5 text-xs font-bold text-slate-600 transition-colors hover:bg-slate-50 disabled:opacity-30 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900"
                            disabled={suite.page <= 1}
                            on:click={() =>
                              loadSuitePage(suite.id, suite.page - 1)}
                          >
                            <ChevronLeft size={14} />
                            {$_("pagination__previous")}
                          </button>
                          <button
                            class="flex items-center gap-1 rounded-md border border-slate-200 px-3 py-1.5 text-xs font-bold text-slate-600 transition-colors hover:bg-slate-50 disabled:opacity-30 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900"
                            disabled={!suite.hasMore}
                            on:click={() =>
                              loadSuitePage(suite.id, suite.page + 1)}
                          >
                            {$_("pagination__next")}
                            <ChevronRight size={14} />
                          </button>
                        {/if}
                      </div>
                    </div>
                  </div>
                </section>
              {/each}
            </div>
          {/if}
          <!-- ══ END VIEW SWITCH ══ -->
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
          title={$_("tc__title_resize")}
          on:pointerdown={startPanelResize}
        ></button>
        <div
          class="sticky top-0 z-10 border-b border-gray-200 bg-white px-5 py-4 dark:border-gray-800 dark:bg-gray-950"
        >
          <div class="flex items-start gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="font-mono text-sm font-bold text-slate-400">
                  TC-{selectedCase.test_no}
                </p>
              </div>
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
              title={$_("tc__title_close_details")}
              on:click={() => (showDetail = false)}
            >
              <X size={18} />
            </button>
          </div>

          <div class="mt-4 flex flex-wrap items-center gap-2">
            <button
              class="flex h-9 items-center gap-2 rounded-lg bg-indigo-600 px-3 text-sm font-black text-white hover:bg-indigo-500 transition-colors shadow-sm"
              title={$_("tc__title_create_task")}
              on:click={() => handleConvertToTask(selectedCase.id)}
            >
              <Plus size={16} />
              Create Task
            </button>
            {#if isAuthorized}
              <button
                class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-500 hover:bg-slate-200 transition-colors dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                title={$_("tc__title_edit")}
                on:click={() =>
                  openTestCaseEditor(selectedSuite.id, selectedCase.id)}
              >
                <Edit3 size={17} />
              </button>
              <button
                class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-500 hover:bg-slate-200 transition-colors dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                title={$_("tc__title_delete")}
                on:click={() => deleteTestCase(selectedCase.id)}
              >
                <Trash2 size={17} />
              </button>
            {/if}
          </div>

          <div
            class="mt-4 flex gap-6 border-b border-gray-200 text-sm font-black text-slate-500 dark:border-gray-800"
          >
            <button
              class="pb-2 transition-colors {sidebarTab === 'general'
                ? 'border-b-2 border-indigo-600 text-slate-800 dark:text-white'
                : 'hover:text-slate-700 dark:hover:text-gray-300'}"
              on:click={() => (sidebarTab = "general")}
            >
              General
            </button>
            <button
              class="pb-2 transition-colors {sidebarTab === 'properties'
                ? 'border-b-2 border-indigo-600 text-slate-800 dark:text-white'
                : 'hover:text-slate-700 dark:hover:text-gray-300'}"
              on:click={() => (sidebarTab = "properties")}
            >
              Properties
            </button>
            <button
              class="pb-2 transition-colors {sidebarTab === 'notes'
                ? 'border-b-2 border-indigo-600 text-slate-800 dark:text-white'
                : 'hover:text-slate-700 dark:hover:text-gray-300'}"
              on:click={() => (sidebarTab = "notes")}
            >
              Notes
            </button>
            <button
              class="pb-2 transition-colors {sidebarTab === 'attachments'
                ? 'border-b-2 border-indigo-600 text-slate-800 dark:text-white'
                : 'hover:text-slate-700 dark:hover:text-gray-300'}"
              on:click={() => (sidebarTab = "attachments")}
            >
              Attachments
            </button>
          </div>
        </div>

        {#if sidebarTab === "general"}
          <div class="space-y-7 px-5 py-6">
            <!-- Description -->
            <section>
              <div class="flex items-center justify-between group">
                <h3
                  class="text-sm font-black text-slate-700 dark:text-gray-200"
                >
                  Description
                </h3>
                {#if isAuthorized && editingField !== "description"}
                  <button
                    class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                    on:click={() =>
                      startEditing("description", selectedCase.description)}
                  >
                    <Edit3 size={14} />
                  </button>
                {/if}
              </div>
              {#if editingField === "description"}
                <div class="mt-2 space-y-2">
                  <textarea
                    class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                    rows="3"
                    bind:value={tempFieldValue}
                    use:focusOnInit
                  ></textarea>
                  <div class="flex justify-end gap-2">
                    <button
                      class="p-1 text-slate-400 hover:text-rose-500"
                      on:click={() => (editingField = null)}
                      ><X size={16} /></button
                    >
                    <button
                      class="p-1 text-indigo-600 hover:text-indigo-500"
                      on:click={() => saveField("description")}
                      disabled={isSavingField}><Check size={16} /></button
                    >
                  </div>
                </div>
              {:else}
                <p
                  class="mt-2 text-sm leading-6 text-slate-700 dark:text-gray-300"
                >
                  {selectedCase.description || "No description"}
                </p>
              {/if}
            </section>

            <!-- Pre & Post Conditions -->
            <section class="grid grid-cols-1 gap-5">
              <!-- Pre-conditions -->
              <div>
                <div class="flex items-center justify-between group">
                  <h3
                    class="text-sm font-black text-slate-700 dark:text-gray-200"
                  >
                    Pre-conditions
                  </h3>
                  {#if isAuthorized && editingField !== "preconditions"}
                    <button
                      class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                      on:click={() =>
                        startEditing(
                          "preconditions",
                          selectedCase.preconditions || "",
                        )}
                    >
                      <Edit3 size={14} />
                    </button>
                  {/if}
                </div>
                {#if editingField === "preconditions"}
                  <div class="mt-2 space-y-2">
                    <textarea
                      class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                      rows="2"
                      bind:value={tempFieldValue}
                      use:focusOnInit
                    ></textarea>
                    <div class="flex justify-end gap-2">
                      <button
                        class="p-1 text-slate-400 hover:text-rose-500"
                        on:click={() => (editingField = null)}
                        ><X size={16} /></button
                      >
                      <button
                        class="p-1 text-indigo-600 hover:text-indigo-500"
                        on:click={() => saveField("preconditions")}
                        disabled={isSavingField}><Check size={16} /></button
                      >
                    </div>
                  </div>
                {:else}
                  <p class="mt-2 text-sm font-medium text-slate-500">
                    {selectedCase.preconditions || "Not set"}
                  </p>
                {/if}
              </div>

              <!-- Post-conditions -->
              <div>
                <div class="flex items-center justify-between group">
                  <h3
                    class="text-sm font-black text-slate-700 dark:text-gray-200"
                  >
                    Post-conditions
                  </h3>
                  {#if isAuthorized && editingField !== "postconditions"}
                    <button
                      class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                      on:click={() =>
                        startEditing(
                          "postconditions",
                          selectedCase.postconditions || "",
                        )}
                    >
                      <Edit3 size={14} />
                    </button>
                  {/if}
                </div>
                {#if editingField === "postconditions"}
                  <div class="mt-2 space-y-2">
                    <textarea
                      class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                      rows="2"
                      bind:value={tempFieldValue}
                      use:focusOnInit
                    ></textarea>
                    <div class="flex justify-end gap-2">
                      <button
                        class="p-1 text-slate-400 hover:text-rose-500"
                        on:click={() => (editingField = null)}
                        ><X size={16} /></button
                      >
                      <button
                        class="p-1 text-indigo-600 hover:text-indigo-500"
                        on:click={() => saveField("postconditions")}
                        disabled={isSavingField}><Check size={16} /></button
                      >
                    </div>
                  </div>
                {:else}
                  <p class="mt-2 text-sm font-medium text-slate-500">
                    {selectedCase.postconditions || "Not set"}
                  </p>
                {/if}
              </div>
            </section>

            <!-- Input, Expected, Actual Results -->
            <section class="space-y-5">
              <!-- Input -->
              <div>
                <div class="flex items-center justify-between group">
                  <h3
                    class="text-sm font-black text-slate-700 dark:text-gray-200"
                  >
                    Input
                  </h3>
                  {#if isAuthorized && editingField !== "input"}
                    <button
                      class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                      on:click={() =>
                        startEditing("input", selectedCase.input || "")}
                    >
                      <Edit3 size={14} />
                    </button>
                  {/if}
                </div>
                {#if editingField === "input"}
                  <div class="mt-2 space-y-2">
                    <textarea
                      class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                      rows="2"
                      bind:value={tempFieldValue}
                      use:focusOnInit
                    ></textarea>
                    <div class="flex justify-end gap-2">
                      <button
                        class="p-1 text-slate-400 hover:text-rose-500"
                        on:click={() => (editingField = null)}
                        ><X size={16} /></button
                      >
                      <button
                        class="p-1 text-indigo-600 hover:text-indigo-500"
                        on:click={() => saveField("input")}
                        disabled={isSavingField}><Check size={16} /></button
                      >
                    </div>
                  </div>
                {:else}
                  <p
                    class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap"
                  >
                    {selectedCase.input || "No input data"}
                  </p>
                {/if}
              </div>

              <!-- Expected Result -->
              <div>
                <div class="flex items-center justify-between group">
                  <h3
                    class="text-sm font-black text-slate-700 dark:text-gray-200"
                  >
                    Expected Result
                  </h3>
                  {#if isAuthorized && editingField !== "expected_result"}
                    <button
                      class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                      on:click={() =>
                        startEditing(
                          "expected_result",
                          selectedCase.expected_result || "",
                        )}
                    >
                      <Edit3 size={14} />
                    </button>
                  {/if}
                </div>
                {#if editingField === "expected_result"}
                  <div class="mt-2 space-y-2">
                    <textarea
                      class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                      rows="2"
                      bind:value={tempFieldValue}
                      use:focusOnInit
                    ></textarea>
                    <div class="flex justify-end gap-2">
                      <button
                        class="p-1 text-slate-400 hover:text-rose-500"
                        on:click={() => (editingField = null)}
                        ><X size={16} /></button
                      >
                      <button
                        class="p-1 text-indigo-600 hover:text-indigo-500"
                        on:click={() => saveField("expected_result")}
                        disabled={isSavingField}><Check size={16} /></button
                      >
                    </div>
                  </div>
                {:else}
                  <p
                    class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap"
                  >
                    {selectedCase.expected_result || "No expected result"}
                  </p>
                {/if}
              </div>

              <!-- Actual Result -->
              <div>
                <div class="flex items-center justify-between group">
                  <h3
                    class="text-sm font-black text-slate-700 dark:text-gray-200"
                  >
                    Actual Result
                  </h3>
                  {#if isAuthorized && editingField !== "actual_result"}
                    <button
                      class="p-1 text-slate-400 hover:text-slate-600 dark:hover:text-gray-200 transition-colors"
                      on:click={() =>
                        startEditing(
                          "actual_result",
                          selectedCase.actual_result || "",
                        )}
                    >
                      <Edit3 size={14} />
                    </button>
                  {/if}
                </div>
                {#if editingField === "actual_result"}
                  <div class="mt-2 space-y-2">
                    <textarea
                      class="w-full rounded-lg border border-indigo-500 bg-transparent p-2 text-sm text-slate-700 dark:text-gray-200 focus:ring-1 focus:ring-indigo-500 outline-none"
                      rows="2"
                      bind:value={tempFieldValue}
                      use:focusOnInit
                    ></textarea>
                    <div class="flex justify-end gap-2">
                      <button
                        class="p-1 text-slate-400 hover:text-rose-500"
                        on:click={() => (editingField = null)}
                        ><X size={16} /></button
                      >
                      <button
                        class="p-1 text-indigo-600 hover:text-indigo-500"
                        on:click={() => saveField("actual_result")}
                        disabled={isSavingField}><Check size={16} /></button
                      >
                    </div>
                  </div>
                {:else}
                  <p
                    class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap"
                  >
                    {selectedCase.actual_result || "No actual result"}
                  </p>
                {/if}
              </div>
            </section>

            <section>
              <div class="mb-5 flex items-center justify-between">
                <h3
                  class="text-sm font-black text-slate-700 dark:text-gray-200"
                >
                  Steps
                </h3>
                <div class="flex items-center gap-4">
                  {#if isAuthorized}
                    {#if !isEditingSidebarSteps}
                      <button
                        class="flex items-center gap-1.5 text-xs font-bold text-indigo-600 hover:text-indigo-500 transition-colors"
                        on:click={() => (isEditingSidebarSteps = true)}
                      >
                        <Edit3 size={14} />
                        Edit
                      </button>
                    {:else}
                      <div class="flex items-center gap-3">
                        <div class="property-select min-w-[96px]">
                          <SearchableSelect
                            id="sidebar-step-format"
                            bind:value={sidebarStepFormat}
                            options={[
                              { value: "classic", label: "Classic" },
                              { value: "gherkin", label: "Gherkin" },
                            ]}
                            showSearch={false}
                            minimal={true}
                          />
                        </div>
                        <button
                          class="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-black text-white hover:bg-indigo-500 disabled:opacity-50 dark:bg-indigo-500 dark:hover:bg-indigo-400"
                          on:click={saveSidebarSteps}
                          disabled={isSavingSidebarSteps}
                        >
                          <Save size={14} />
                          {isSavingSidebarSteps ? "Saving..." : "Save"}
                        </button>
                        <button
                          class="text-xs font-bold text-gray-500 hover:text-gray-700 transition-colors"
                          on:click={() => {
                            isEditingSidebarSteps = false;
                            // Reset local steps to match selected case
                            sidebarStepFormat =
                              selectedCase.step_format || "classic";
                            sidebarSteps =
                              sidebarStepFormat === "gherkin"
                                ? [...(selectedCase.gherkin_steps || [])]
                                : [
                                    ...(selectedCase.classic_steps ||
                                      selectedCase.steps ||
                                      []),
                                  ];
                          }}
                        >
                          Cancel
                        </button>
                      </div>
                    {/if}
                  {/if}
                </div>
              </div>

              <TestCaseStepList
                bind:steps={sidebarSteps}
                format={sidebarStepFormat}
                readOnly={!isEditingSidebarSteps}
              />
            </section>
          </div>
        {:else if sidebarTab === "properties"}
          <div class="space-y-6 px-5 py-6">
            <section>
              <h3
                class="mb-3 text-sm font-black text-slate-700 dark:text-gray-200"
              >
                Priority
              </h3>
              <div class="property-select w-full">
                <SearchableSelect
                  id="sidebar-prop-priority-update"
                  value={selectedCase.priority}
                  options={priorityOptions}
                  showSearch={false}
                  disabled={!isAuthorized}
                  on:select={(e) => updatePriority(selectedCase, e.detail)}
                />
              </div>
            </section>

            <section>
              <h3
                class="mb-3 text-sm font-black text-slate-700 dark:text-gray-200"
              >
                Status
              </h3>
              <div class="property-select w-full">
                <SearchableSelect
                  id="sidebar-prop-status-update"
                  value={selectedCase.status}
                  options={statusOptions}
                  showSearch={false}
                  disabled={!isAuthorized}
                  on:select={(e) => updateStatus(selectedCase, e.detail)}
                />
              </div>
            </section>

            <section>
              <h3
                class="mb-3 text-sm font-black text-slate-700 dark:text-gray-200"
              >
                Fix Status
              </h3>
              <div class="property-select w-full">
                <SearchableSelect
                  id="sidebar-prop-fixed-update"
                  value={selectedCase.fixed}
                  options={fixedOptions}
                  showSearch={false}
                  on:select={(e) => updateFixed(selectedCase, e.detail)}
                />
              </div>
            </section>

            <section>
              <h3
                class="mb-3 text-sm font-black text-slate-700 dark:text-gray-200"
              >
                Assign Developer
              </h3>
              <div class="property-select w-full">
                <SearchableSelect
                  id="sidebar-prop-assigndev-update"
                  value={selectedCase.assign_dev || "unassigned"}
                  options={availableAssigneeOptions}
                  showSearch={true}
                  on:select={(e) => updateAssignDev(selectedCase, e.detail)}
                />
              </div>
            </section>

            <section>
              <h3
                class="mb-3 text-sm font-black text-slate-700 dark:text-gray-200"
              >
                Assign Tester
              </h3>
              <div class="property-select w-full">
                <SearchableSelect
                  id="sidebar-prop-assigntester-update"
                  value={selectedCase.assignee || "unassigned"}
                  options={assigneeOptions}
                  showSearch={true}
                  disabled={!isAuthorized}
                  on:select={(e) => updateAssignTester(selectedCase, e.detail)}
                />
              </div>
            </section>
          </div>
        {:else if sidebarTab === "notes"}
          <div class="space-y-6 px-5 py-6">
            <section>
              <div class="flex items-center justify-between mb-2">
                <h3
                  class="text-sm font-black text-slate-700 dark:text-gray-200"
                >
                  Dev Notes
                </h3>
                <button
                  class="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-black text-white hover:bg-indigo-500 disabled:opacity-50 dark:bg-indigo-500 dark:hover:bg-indigo-400 transition-all active:scale-95 shadow-sm shadow-indigo-200 dark:shadow-none"
                  on:click={handleUpdateNotes}
                  disabled={isSavingNotes}
                >
                  <Save size={14} />
                  {isSavingNotes ? "Saving..." : "Save"}
                </button>
              </div>
              <textarea
                class="w-full rounded-xl border border-gray-200 bg-slate-50 p-3 text-sm text-slate-700 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 dark:border-gray-700 dark:!bg-transparent dark:text-gray-200"
                rows="6"
                placeholder={$_("tc__ph_dev_notes")}
                bind:value={devNote}
              ></textarea>
            </section>

            <section>
              <div class="flex items-center justify-between mb-2">
                <h3
                  class="text-sm font-black text-slate-700 dark:text-gray-200"
                >
                  Tester Notes
                </h3>
                {#if isAuthorized}
                  <button
                    class="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-black text-white hover:bg-indigo-500 disabled:opacity-50 dark:bg-indigo-500 dark:hover:bg-indigo-400 transition-all active:scale-95 shadow-sm shadow-indigo-200 dark:shadow-none"
                    on:click={handleUpdateNotes}
                    disabled={isSavingNotes}
                  >
                    <Save size={14} />
                    {isSavingNotes ? "Saving..." : "Save"}
                  </button>
                {/if}
              </div>
              <textarea
                class="w-full rounded-xl border border-gray-200 bg-slate-50 p-3 text-sm text-slate-700 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 dark:border-gray-700 dark:!bg-transparent dark:text-gray-200 disabled:bg-gray-100/50 dark:disabled:bg-gray-900/50"
                rows="6"
                placeholder={isAuthorized
                  ? "Add tester notes..."
                  : "Only QA Testers can add notes here"}
                bind:value={testNote}
                disabled={!isAuthorized}
              ></textarea>
            </section>
          </div>
        {:else if sidebarTab === "attachments"}
          <div class="px-5 py-6">
            {#if selectedCase.attachments && selectedCase.attachments.length > 0}
              <div class="grid grid-cols-2 gap-3">
                {#each selectedCase.attachments as attachment, i}
                  <button
                    class="group relative aspect-square overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:border-gray-800 dark:bg-gray-900/50"
                    on:click={() => openLightbox(selectedCase.attachments, i)}
                  >
                    <img
                      src={getAttachmentUrl(attachment.file_key)}
                      alt={attachment.filename}
                      class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
                    />
                    <div
                      class="absolute inset-0 bg-black/0 transition-colors group-hover:bg-black/20"
                    ></div>
                    <div
                      class="absolute bottom-2 right-2 rounded-lg bg-black/60 p-1.5 text-white opacity-0 transition-opacity group-hover:opacity-100 backdrop-blur-sm"
                    >
                      <ImageIcon size={14} />
                    </div>
                  </button>
                {/each}
              </div>
            {:else}
              <div
                class="flex flex-col items-center justify-center py-20 text-center"
              >
                <div
                  class="mb-4 rounded-full bg-slate-100 p-4 text-slate-400 dark:bg-gray-800 dark:text-gray-500"
                >
                  <Paperclip size={32} />
                </div>
                <h3
                  class="text-sm font-black text-slate-700 dark:text-gray-200"
                >
                  No attachments yet
                </h3>
                <p class="mt-1 text-xs text-slate-500">
                  Images added to this test case will appear here
                </p>
              </div>
            {/if}
          </div>
        {/if}
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
          <span class="text-gray-400">{$_("tc__create_test_case")}</span>
        </div>
        <button
          type="button"
          class="p-1.5 text-gray-400 hover:text-white hover:!bg-transparent rounded-md transition-all"
          title={$_("tc__close")}
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
              placeholder={$_("tc__ph_title")}
            />
          </div>

          <div class="min-h-[120px]">
            <textarea
              bind:value={testCaseForm.description}
              class="min-h-[120px] w-full resize-none !bg-transparent px-0 text-[15px] leading-relaxed text-gray-300 placeholder:text-gray-600 border-none outline-none"
              placeholder={$_("tc__ph_details")}
            ></textarea>
          </div>

          <div class="flex flex-wrap items-center gap-2 pt-2 pb-4">
            <div class="property-select min-w-[150px]">
              <SearchableSelect
                id="test-case-suite"
                bind:value={testCaseForm.suiteId}
                options={suiteOptions}
                placeholder={$_("tc__ph_search_suites")}
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
                placeholder={$_("tc__ph_search_assignees")}
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
                  >{$_("tc__preconditions")}</span
                >
                <textarea
                  bind:value={testCaseForm.preconditions}
                  class="mt-3 min-h-28 w-full resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                  placeholder={$_("tc__ph_preconditions")}
                ></textarea>
              </label>

              <label>
                <span
                  class="text-[12px] font-black uppercase tracking-widest text-gray-500"
                  >{$_("tc__postconditions")}</span
                >
                <textarea
                  bind:value={testCaseForm.postconditions}
                  class="mt-3 min-h-28 w-full resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
                  placeholder={$_("tc__ph_postconditions")}
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
                  <span>{$_("tc__raw")}</span>
                  <span class="text-indigo-400">{$_("tc__steps")}</span>
                </div>
              {/if}
            </div>

            {#if stepFormat === "classic"}
              <TestCaseStepList
                bind:steps={classicSteps}
                format="classic"
                readOnly={false}
              />
            {:else}
              <TestCaseStepList
                bind:steps={testCaseSteps}
                format="gherkin"
                readOnly={false}
              />
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
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="new-suite-name"
          type="text"
          bind:value={newSuiteName}
          class="mt-3 w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-800 outline-none focus:border-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:focus:border-indigo-400"
          placeholder={$_("tc__ph_suite_name")}
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
          >{$_("tc__suite_name")}</label
        >
        <!-- svelte-ignore a11y_autofocus -->
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
          on:click={() => (editingSuite = null)}>{$_("tc__cancel")}</button
        >
        <button
          class="rounded-xl bg-indigo-600 px-6 py-2 text-sm font-black text-white hover:bg-indigo-500 disabled:opacity-50"
          disabled={!editingSuite.title.trim()}
          on:click={handleUpdateSuite}>{$_("tc__save_changes")}</button
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

      {#if suiteToDelete.id !== "unassigned"}
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
      {:else}
        <div
          class="mt-6 p-4 rounded-xl bg-rose-50 dark:bg-rose-900/10 border border-rose-100 dark:border-rose-900/20"
        >
          <div
            class="flex items-center gap-3 text-rose-600 dark:text-rose-400 mb-2"
          >
            <AlertCircle size={18} />
            <p class="text-sm font-bold">{$_("tc__critical_action")}</p>
          </div>
          <p class="text-xs text-slate-600 dark:text-gray-400 leading-relaxed">
            Deleting the <span class="font-bold">Unassigned</span> collection will
            permanently remove all test cases that do not belong to any suite. This
            action cannot be undone.
          </p>
        </div>
      {/if}

      <div class="mt-8 flex items-center justify-end gap-3">
        <button
          class="rounded-xl px-4 py-2 text-sm font-bold text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => (showDeleteSuiteModal = false)}>{$_("tc__cancel")}</button
        >
        <button
          class="rounded-xl bg-rose-600 px-6 py-2 text-sm font-black text-white hover:bg-rose-500"
          on:click={handleDeleteSuite}>{$_("tc__delete_suite")}</button
        >
      </div>
    </div>
  </div>
{/if}

{#if showNewTestRunModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="w-full max-w-2xl rounded-2xl bg-white shadow-2xl dark:bg-gray-900 flex flex-col h-[85vh]"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-8 pt-8 pb-4 shrink-0">
        <h2 class="text-xl font-black text-slate-800 dark:text-white">
          New test run
        </h2>
        <button
          class="grid h-8 w-8 place-items-center rounded-lg text-slate-400 hover:bg-slate-100 dark:hover:bg-gray-800"
          on:click={() => (showNewTestRunModal = false)}
        >
          <X size={18} />
        </button>
      </div>

      <div class="overflow-y-auto flex-1 px-8 pb-2 space-y-5">
        <!-- Name -->
        <div>
          <input
            type="text"
            bind:value={newTestRunForm.name}
            class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-800 outline-none focus:border-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:focus:border-indigo-400"
            placeholder={$_("tc__ph_run_name")}
          />
        </div>

        <!-- Description -->
        <div>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label
            class="block text-sm font-black text-slate-700 dark:text-gray-200 mb-2"
            >{$_("tc__description")}</label
          >
          <textarea
            bind:value={newTestRunForm.description}
            rows="2"
            class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-800 outline-none focus:border-indigo-500 resize-none dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:focus:border-indigo-400"
            placeholder=""
          ></textarea>
        </div>

        <!-- Assignee + OS row -->
        <div class="grid grid-cols-2 gap-4">
          <div>
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label
              class="block text-xs font-black uppercase tracking-widest text-slate-500 dark:text-gray-400 mb-2"
              >{$_("tc__default_assignee")}</label
            >
            <div class="property-select w-full">
              <SearchableSelect
                id="new-test-run-assignee"
                value={newTestRunForm.defaultAssignee}
                options={testRunAssigneeOptions}
                showSearch={true}
                on:select={(e) => (newTestRunForm.defaultAssignee = e.detail)}
              />
            </div>
          </div>
          <div>
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <label
              class="block text-xs font-black uppercase tracking-widest text-slate-500 dark:text-gray-400 mb-2"
              >{$_("tc__operating_system")}</label
            >
            <div class="property-select w-full">
              <SearchableSelect
                id="new-test-run-os"
                value={newTestRunForm.operatingSystem}
                options={osOptions}
                showSearch={false}
                on:select={(e) => (newTestRunForm.operatingSystem = e.detail)}
              />
            </div>
          </div>
        </div>

        <!-- Test cases -->
        <div>
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
              Test cases
            </h3>
            <span class="text-xs text-slate-400"
              >{selectedTestCaseIds.size} selected</span
            >
          </div>

          <!-- Search -->
          <div class="mb-3 flex items-center gap-2">
            <div
              class="group flex h-10 flex-1 rounded-xl border border-slate-200 bg-white shadow-sm transition-all focus-within:border-indigo-500 focus-within:ring-4 focus-within:ring-indigo-500/10 dark:border-gray-700 dark:bg-gray-900 dark:focus-within:border-indigo-400"
            >
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="flex min-w-0 flex-1 items-center gap-2.5 px-3.5">
                <Search
                  size={16}
                  class="shrink-0 text-slate-400 group-focus-within:text-indigo-500"
                />
                <input
                  bind:value={testRunSearchQuery}
                  class="min-w-0 flex-1 border-0 bg-transparent text-sm font-bold text-slate-700 outline-none placeholder:text-slate-400 dark:!bg-transparent dark:text-gray-200"
                  placeholder={$_("tc__ph_search")}
                />
                {#if testRunSearchQuery}
                  <button
                    on:click={() => (testRunSearchQuery = "")}
                    class="p-1 hover:bg-slate-100 rounded-full dark:hover:bg-gray-800"
                  >
                    <X size={14} class="text-slate-400" />
                  </button>
                {/if}
              </label>
              <div
                class="flex h-full w-36 items-center rounded-r-xl border-l border-slate-200 bg-slate-50/50 dark:border-gray-700 dark:bg-gray-800/50"
              >
                <SearchableSelect
                  id="test-run-search-field"
                  bind:value={testRunSearchField}
                  options={fieldOptions}
                  showSearch={false}
                  minimal={true}
                />
              </div>
            </div>
            <div class="relative shrink-0">
              <button
                class="flex h-10 items-center gap-2 rounded-xl px-4 text-[13px] font-black transition-all active:scale-95 {hasTestRunFilters
                  ? 'bg-indigo-100 text-indigo-700 hover:bg-indigo-200 dark:bg-indigo-500/20 dark:text-indigo-300'
                  : 'bg-slate-100 text-slate-600 hover:bg-slate-200 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700'}"
                on:click={() => {
                  showTestRunFilterDropdown = !showTestRunFilterDropdown;
                  testRunActiveFilterProperty = null;
                }}
              >
                <Plus size={15} />
                Add filter
                {#if hasTestRunFilters}
                  <span
                    class="ml-0.5 h-4 w-4 rounded-full bg-indigo-600 text-[10px] font-black text-white flex items-center justify-center"
                  >
                    {Object.values(testRunActiveFilters).reduce(
                      (s, v) => s + v.length,
                      0,
                    )}
                  </span>
                {/if}
              </button>

              {#if showTestRunFilterDropdown}
                <!-- svelte-ignore a11y_consider_explicit_label -->
                <button
                  class="fixed inset-0 z-[60] h-full w-full cursor-default border-none bg-transparent"
                  on:click={() => (showTestRunFilterDropdown = false)}
                ></button>
                <div
                  class="absolute right-0 top-full z-[70] mt-1 w-56 overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xl dark:border-gray-700 dark:bg-gray-800"
                >
                  {#if !testRunActiveFilterProperty}
                    <div class="flex flex-col py-1">
                      <div
                        class="px-4 py-2 text-[10px] font-black uppercase tracking-wider text-slate-400"
                      >
                        Filter by
                      </div>
                      {#each testRunFilterProperties as prop}
                        <button
                          class="flex w-full items-center justify-between px-4 py-2.5 text-left text-sm font-bold text-slate-700 hover:bg-slate-50 dark:text-gray-200 dark:hover:bg-gray-700/50 transition-colors"
                          on:click={() =>
                            (testRunActiveFilterProperty = prop.id)}
                        >
                          <span>{prop.label}</span>
                          <div class="flex items-center gap-1.5">
                            {#if testRunActiveFilters[prop.id]?.length > 0}
                              <span class="text-xs font-black text-indigo-500"
                                >{testRunActiveFilters[prop.id].length}</span
                              >
                            {/if}
                            <ChevronRight size={14} class="opacity-30" />
                          </div>
                        </button>
                      {/each}
                      {#if hasTestRunFilters}
                        <div
                          class="h-px bg-slate-100 dark:bg-gray-700 my-1"
                        ></div>
                        <button
                          class="px-4 py-2 text-sm font-black text-indigo-600 hover:bg-indigo-50 dark:text-indigo-400 dark:hover:bg-indigo-500/10 text-center w-full transition-colors"
                          on:click={() => {
                            testRunActiveFilters = { priority: [], status: [] };
                            showTestRunFilterDropdown = false;
                          }}
                        >
                          Clear all filters
                        </button>
                      {/if}
                    </div>
                  {:else}
                    <div class="flex flex-col py-1 max-h-72 overflow-y-auto">
                      <div class="flex items-center gap-2 px-3 py-2">
                        <button
                          class="p-1 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg dark:hover:bg-gray-700 transition-colors"
                          on:click={() => (testRunActiveFilterProperty = null)}
                        >
                          <ChevronLeft size={16} />
                        </button>
                        <span
                          class="text-[10px] font-black uppercase tracking-wider text-slate-400"
                        >
                          {testRunFilterProperties.find(
                            (p) => p.id === testRunActiveFilterProperty,
                          )?.label}
                        </span>
                      </div>
                      <div
                        class="h-px bg-slate-100 dark:bg-gray-700 mb-1"
                      ></div>
                      {#if testRunActiveFilterProperty === "priority"}
                        {#each priorityOptions as opt}
                          <label
                            class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                          >
                            <input
                              type="checkbox"
                              class="rounded border-slate-300 accent-indigo-600 w-4 h-4"
                              checked={testRunActiveFilters.priority.includes(
                                opt.value,
                              )}
                              on:change={() =>
                                toggleTestRunFilter("priority", opt.value)}
                            />
                            <span
                              class="text-sm font-bold text-slate-700 dark:text-gray-200"
                              >{opt.label}</span
                            >
                          </label>
                        {/each}
                      {:else if testRunActiveFilterProperty === "status"}
                        {#each statusOptions as opt}
                          <label
                            class="flex cursor-pointer items-center gap-3 px-4 py-2.5 hover:bg-slate-50 dark:hover:bg-gray-700/50 transition-colors"
                          >
                            <input
                              type="checkbox"
                              class="rounded border-slate-300 accent-indigo-600 w-4 h-4"
                              checked={testRunActiveFilters.status.includes(
                                opt.value,
                              )}
                              on:change={() =>
                                toggleTestRunFilter("status", opt.value)}
                            />
                            <span
                              class="text-sm font-bold text-slate-700 dark:text-gray-200"
                              >{opt.label}</span
                            >
                          </label>
                        {/each}
                      {/if}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          </div>

          <!-- Active filter chips -->
          {#if hasTestRunFilters}
            <div class="flex flex-wrap gap-1.5 mb-3">
              {#each Object.entries(testRunActiveFilters) as [prop, values]}
                {#each values as val}
                  <span
                    class="flex items-center gap-1 rounded-lg bg-indigo-50 border border-indigo-100 px-2.5 py-1 text-[12px] font-black text-indigo-700 dark:bg-indigo-500/10 dark:border-indigo-500/20 dark:text-indigo-300"
                  >
                    <span class="opacity-60"
                      >{testRunFilterProperties.find((p) => p.id === prop)
                        ?.label}:</span
                    >
                    <span>{val}</span>
                    <button
                      class="ml-0.5 rounded-full p-0.5 hover:bg-indigo-200 dark:hover:bg-indigo-500/30"
                      on:click={() => toggleTestRunFilter(prop, val)}
                    >
                      <X size={11} strokeWidth={3} />
                    </button>
                  </span>
                {/each}
              {/each}
            </div>
          {/if}

          <!-- Suite + Cases table -->
          <div
            class="rounded-xl border border-slate-200 dark:border-gray-700 overflow-hidden"
          >
            <div
              class="text-[11px] font-black uppercase tracking-widest text-slate-400 bg-slate-50 dark:bg-gray-800/50 px-4 py-2 border-b border-slate-200 dark:border-gray-700"
            >
              <span>{$_("tc__test_case")}</span>
            </div>
            <div
              class="max-h-56 overflow-y-auto divide-y divide-slate-100 dark:divide-gray-800"
            >
              {#each suites.filter((s) => s.cases.length > 0) as suite}
                {@const filteredCases = suite.cases.filter((c) => {
                  if (testRunSearchQuery) {
                    const q = testRunSearchQuery.toLowerCase();
                    const matchSearch =
                      testRunSearchField === "all"
                        ? c.title.toLowerCase().includes(q) ||
                          String(c.test_no).includes(q) ||
                          (c.description?.toLowerCase().includes(q) ?? false)
                        : testRunSearchField === "test_no"
                          ? String(c.test_no).includes(q)
                          : testRunSearchField === "name"
                            ? c.title.toLowerCase().includes(q)
                            : testRunSearchField === "description"
                              ? (c.description?.toLowerCase().includes(q) ??
                                false)
                              : c.title.toLowerCase().includes(q);
                    if (!matchSearch) return false;
                  }
                  if (
                    testRunActiveFilters.priority.length > 0 &&
                    !testRunActiveFilters.priority.includes(c.priority)
                  )
                    return false;
                  if (
                    testRunActiveFilters.status.length > 0 &&
                    !testRunActiveFilters.status.includes(c.status)
                  )
                    return false;

                  return true;
                })}
                {#if filteredCases.length > 0}
                  <!-- Suite header row -->
                  <div
                    class="grid grid-cols-[1fr_auto] items-center px-4 py-2 bg-slate-50 dark:bg-gray-800/30"
                  >
                    <label
                      class="flex items-center gap-2.5 cursor-pointer select-none"
                    >
                      <input
                        type="checkbox"
                        class="rounded border-slate-300 accent-indigo-600"
                        checked={suite.cases.every((c) =>
                          selectedTestCaseIds.has(c.id),
                        )}
                        indeterminate={suite.cases.some((c) =>
                          selectedTestCaseIds.has(c.id),
                        ) &&
                          !suite.cases.every((c) =>
                            selectedTestCaseIds.has(c.id),
                          )}
                        on:change={() => toggleSuiteSelection(suite)}
                      />
                      <span
                        class="text-xs font-black text-slate-600 dark:text-gray-300"
                        >{suite.title}</span
                      >
                      <span class="text-[10px] text-slate-400"
                        >{suite.totalCount ?? suite.cases.length}</span
                      >
                    </label>
                  </div>
                  <!-- Cases -->
                  {#each filteredCases as tc}
                    <div
                      class="flex items-center px-4 py-1.5 hover:bg-slate-50 dark:hover:bg-gray-800/30"
                    >
                      <!-- svelte-ignore a11y_label_has_associated_control -->
                      <label
                        class="flex items-center gap-2.5 cursor-pointer select-none min-w-0 flex-1"
                      >
                        <input
                          type="checkbox"
                          class="shrink-0 rounded border-slate-300 accent-indigo-600"
                          checked={selectedTestCaseIds.has(tc.id)}
                          on:change={() => toggleTestCaseSelection(tc.id)}
                        />
                        {#if tc.test_no}
                          <span
                            class="shrink-0 text-[11px] font-black text-slate-400 dark:text-gray-500"
                            >TC-{tc.test_no}</span
                          >
                        {/if}
                        <span
                          class="text-xs text-slate-600 dark:text-gray-300 truncate"
                          >{tc.title}</span
                        >
                      </label>
                    </div>
                  {/each}
                {/if}
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div
        class="flex items-center justify-end gap-3 px-8 py-5 border-t border-slate-100 dark:border-gray-800 shrink-0"
      >
        <button
          class="rounded-xl px-5 py-2 text-sm font-bold text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
          on:click={() => (showNewTestRunModal = false)}>{$_("tc__cancel")}</button
        >
        <button
          class="rounded-xl bg-indigo-600 px-6 py-2 text-sm font-black text-white hover:bg-indigo-500 disabled:opacity-50"
          disabled={!newTestRunForm.name.trim()}
          on:click={handleCreateTestRun}>{$_("tc__start_run")}</button
        >
      </div>
    </div>
  </div>
{/if}

{#if showImportModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="w-full max-w-sm rounded-2xl bg-white p-8 shadow-2xl dark:bg-gray-900"
    >
      <div class="flex flex-col items-center text-center">
        <div class="relative mb-6">
          <svg class="h-20 w-20 transform -rotate-90">
            <circle
              cx="40"
              cy="40"
              r="36"
              stroke="currentColor"
              stroke-width="8"
              fill="transparent"
              class="text-slate-100 dark:text-gray-800"
            />
            <circle
              cx="40"
              cy="40"
              r="36"
              stroke="currentColor"
              stroke-width="8"
              fill="transparent"
              stroke-dasharray={2 * Math.PI * 36}
              stroke-dashoffset={2 *
                Math.PI *
                36 *
                (1 - importProgress.current / (importProgress.total || 1))}
              stroke-linecap="round"
              class="text-indigo-600 transition-all duration-300"
            />
          </svg>
          <div
            class="absolute inset-0 flex items-center justify-center text-sm font-black text-indigo-600"
          >
            {Math.round(
              (importProgress.current / (importProgress.total || 1)) * 100,
            )}%
          </div>
        </div>

        <h3 class="text-lg font-black text-slate-800 dark:text-white">
          Importing Test Cases
        </h3>
        <p class="mt-2 text-sm text-slate-500 dark:text-gray-400">
          {importProgress.current} of {importProgress.total} processed
        </p>

        <div
          class="mt-6 w-full rounded-xl bg-amber-50 p-4 border border-amber-100 dark:bg-amber-900/10 dark:border-amber-900/20"
        >
          <div
            class="flex items-center gap-2 text-amber-600 dark:text-amber-500 mb-1"
          >
            <AlertCircle size={16} />
            <p class="text-xs font-bold uppercase tracking-wider">
              {$_("tc__do_not_refresh")}
            </p>
          </div>
          <p
            class="text-[11px] text-amber-700/80 dark:text-amber-400/80 leading-relaxed"
          >
            {$_("tc__import_warning")}

          </p>
        </div>

        <p
          class="mt-4 truncate w-full text-[10px] font-mono text-slate-400 dark:text-gray-500"
        >
          {importProgress.status}
        </p>
      </div>
    </div>
  </div>
{/if}

{#if lightboxOpen}
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center bg-slate-950/95 backdrop-blur-md"
    on:keydown={handleLightboxKeydown}
    on:wheel|preventDefault={lbZoomIn}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="absolute top-4 right-4 flex items-center gap-1.5 z-10">
      <span
        class="text-white/70 text-sm font-mono bg-black/40 px-2 py-1 rounded"
        >{Math.round(lightboxZoom * 100)}%</span
      >
      {#if lightboxRotation !== 0}
        <span
          class="text-white/70 text-sm font-mono bg-black/40 px-2 py-1 rounded"
          >{lightboxRotation}°</span
        >
      {/if}
      {#if lightboxImages.length > 1}
        <span class="text-white/70 text-sm bg-black/40 px-2 py-1 rounded"
          >{lightboxIndex + 1}/{lightboxImages.length}</span
        >
      {/if}
      <div class="w-px h-5 bg-white/20"></div>
      <button
        type="button"
        on:click={lbZoomIn}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_zoom_in")}><ZoomIn size={18} /></button
      >
      <button
        type="button"
        on:click={lbZoomOut}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_zoom_out")}><ZoomOut size={18} /></button
      >
      <button
        type="button"
        on:click={lbRotateRight}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_rotate")}><RotateCw size={18} /></button
      >
      <button
        type="button"
        on:click={lbResetView}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_reset")}><RotateCcw size={18} /></button
      >
      <div class="w-px h-5 bg-white/20"></div>
      <button
        type="button"
        on:click={() => void lbDownload()}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_download")}><Download size={18} /></button
      >
      <button
        type="button"
        on:click={lbOpenInNewTab}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title={$_("tc__title_open_new_tab")}><ExternalLink size={18} /></button
      >
      <div class="w-px h-5 bg-white/20"></div>
      <button
        type="button"
        on:click={closeLightbox}
        class="p-2 bg-black/40 hover:bg-red-600/80 text-white rounded-lg transition-colors"
        title={$_("tc__close")}><X size={18} /></button
      >
    </div>

    {#if lightboxImages.length > 1}
      <button
        type="button"
        on:click={lbNavigatePrev}
        class="absolute left-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
        title={$_("tc__title_previous")}
      >
        <ChevronLeft size={24} />
      </button>
      <button
        type="button"
        on:click={lbNavigateNext}
        class="absolute right-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
        title={$_("tc__title_next")}
      >
        <ChevronRight size={24} />
      </button>
    {/if}

    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <img
      src={lightboxSrc}
      alt={lightboxAlt}
      class="max-w-[90vw] max-h-[85vh] object-contain select-none {isLightboxDragging
        ? 'cursor-grabbing'
        : 'cursor-grab transition-transform duration-150'}"
      style="transform: scale({lightboxZoom}) translate({lightboxX /
        lightboxZoom}px, {lightboxY /
        lightboxZoom}px) rotate({lightboxRotation}deg);"
      on:mousedown={handleLightboxMouseDown}
      on:mousemove={handleLightboxMouseMove}
      on:mouseup={handleLightboxMouseUp}
      on:mouseleave={handleLightboxMouseUp}
      draggable="false"
      role="presentation"
    />

    {#if lightboxAlt && lightboxAlt !== "Image"}
      <div
        class="absolute bottom-4 left-1/2 -translate-x-1/2 bg-black/50 text-white/80 text-sm px-4 py-1.5 rounded-full"
      >
        {lightboxAlt}
      </div>
    {/if}
  </div>
{/if}

<ConfirmModal
  show={showDeleteRunModal}
  title={$_("tc__confirm_delete_run_title")}
  message={$_("tc__confirm_delete_run_msg")}
  confirmText={$_("tc__confirm_delete")}
  cancelText={$_("tc__cancel")}
  type="danger"
  onConfirm={confirmDeleteTestRun}
  onClose={() => (showDeleteRunModal = false)}
/>

<ConfirmModal
  show={showDeleteModal}
  title={$_("tc__confirm_delete_case_title")}
  message={$_("tc__confirm_delete_case_msg")}
  confirmText={$_("tc__confirm_delete")}
  cancelText={$_("tc__cancel")}
  type="danger"
  onConfirm={confirmDeleteTestCase}
  onClose={() => {
    showDeleteModal = false;
    caseToDeleteId = "";
  }}
/>

{#if importSummary}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="w-full max-w-lg rounded-2xl bg-white p-8 shadow-2xl dark:bg-gray-900"
    >
      <div class="flex items-center gap-3 mb-6">
        <div
          class="grid h-12 w-12 place-items-center rounded-xl bg-indigo-50 text-indigo-600 dark:bg-indigo-900/20 dark:text-indigo-400"
        >
          <FileCheck size={24} />
        </div>
        <div>
          <h2 class="text-xl font-black text-slate-800 dark:text-white">
            Import Summary
          </h2>
          <p class="text-sm text-slate-500 dark:text-gray-400">
            Process completed
          </p>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-4 mb-8">
        <div class="rounded-xl bg-slate-50 p-4 dark:bg-gray-800/50">
          <p
            class="text-[10px] font-black uppercase tracking-wider text-slate-400 mb-1"
          >
            Success
          </p>
          <p class="text-2xl font-black text-emerald-600 dark:text-emerald-400">
            {importSummary.success}
          </p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4 dark:bg-gray-800/50">
          <p
            class="text-[10px] font-black uppercase tracking-wider text-slate-400 mb-1"
          >
            Failed
          </p>
          <p class="text-2xl font-black text-rose-600 dark:text-rose-400">
            {importSummary.failed}
          </p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4 dark:bg-gray-800/50">
          <p
            class="text-[10px] font-black uppercase tracking-wider text-slate-400 mb-1"
          >
            Skipped
          </p>
          <p class="text-2xl font-black text-amber-600 dark:text-amber-400">
            {importSummary.skipped.length}
          </p>
        </div>
      </div>

      {#if importSummary.failedNames.length > 0}
        <div class="mb-6">
          <p class="text-xs font-bold text-rose-600 dark:text-rose-400 mb-2">
            Failed Cases
          </p>
          <div
            class="rounded-xl border border-rose-100 bg-rose-50/50 p-3 dark:border-rose-900/30 dark:bg-rose-900/10"
          >
            {#each importSummary.failedNames.slice(0, 3) as name}
              <div class="flex items-center gap-2 py-1.5">
                <XCircle size={12} class="text-rose-400 shrink-0" />
                <span
                  class="text-[12px] font-semibold text-rose-700 dark:text-rose-300 truncate"
                  >{name}</span
                >
              </div>
            {/each}
            {#if importSummary.failedNames.length > 3}
              <p
                class="text-[11px] font-bold text-rose-400 dark:text-rose-500 mt-1 pl-5"
              >
                ...and {importSummary.failedNames.length - 3} more
              </p>
            {/if}
          </div>
        </div>
      {/if}

      {#if importSummary.skipped.length > 0}
        <div class="mb-6">
          <p class="text-xs font-bold text-amber-600 dark:text-amber-400 mb-2">
            Skipped Rows
          </p>
          <div
            class="rounded-xl border border-amber-100 bg-amber-50/50 p-3 dark:border-amber-900/30 dark:bg-amber-900/10"
          >
            {#each importSummary.skipped.slice(0, 3) as item}
              <div class="flex items-start gap-2 py-1.5">
                <AlertCircle size={12} class="text-amber-400 shrink-0 mt-0.5" />
                <div class="min-w-0">
                  <span
                    class="text-[12px] font-semibold text-amber-700 dark:text-amber-300"
                  >
                    Row {item.row}{item.name ? `: ${item.name}` : ""}
                  </span>
                  <p class="text-[11px] text-amber-500 dark:text-amber-500">
                    {item.reason}
                  </p>
                </div>
              </div>
            {/each}
            {#if importSummary.skipped.length > 3}
              <p
                class="text-[11px] font-bold text-amber-400 dark:text-amber-500 mt-1 pl-5"
              >
                ...and {importSummary.skipped.length - 3} more
              </p>
            {/if}
          </div>
        </div>
      {/if}

      <div class="flex justify-end">
        <button
          class="rounded-xl bg-indigo-600 px-8 py-2 text-sm font-black text-white hover:bg-indigo-500 shadow-lg shadow-indigo-600/20"
          on:click={() => (importSummary = null)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}

<WorkspaceModals
  modals={$modals}
  editingTask={$wsEditingTask}
  assignees={$wsAssignees}
  isOwner={$user?.role === "admin"}
  projectList={$projectList}
  sprints={$sprints}
  allTasksIncludingArchived={$allTasksIncludingArchived}
  qrExportTasks={[]}
  loadingData={$wsLoadingData}
  workerStats={$workerStats}
  projectStats={$projectStats}
  {monthlySummary}
  monthlySummaryRef={null}
  workspaceId={String($page.params.workspace_id || "")}
  newPageSize={20}
  on:addTask={(e) => taskActions.handleAddTask(e)}
  on:cancelEdit={cancelTaskEdit}
  on:addAssignee={(e) => taskActions.handleAddAssignee(e)}
  on:checklistUpdate={(e) => taskActions.handleChecklistUpdate(e)}
  on:closeModal={(e) => ui.closeModal(e.detail)}
  on:closeWorkerManager={() => ui.closeModal("workerManager")}
  on:addWorker={(e) => wsActions.handleAddWorker(e)}
  on:updateWorker={(e) => wsActions.handleUpdateWorker(e)}
  on:deleteWorker={(e) => wsActions.handleDeleteWorker(e)}
  on:closeSprintManager={() => ui.closeModal("sprintManager")}
  on:completeSprint={(e) => sprintActions.handleCompleteSprint(e)}
  on:deleteSprint={(e) => sprintActions.handleDeleteSprint(e)}
  on:moveTasksToSprint={(e) => sprintActions.handleMoveTasksToSprint(e)}
  on:closeProjectManager={() => ui.closeModal("projectManager")}
  on:addProject={(e) => wsActions.handleAddProject(e)}
  on:updateProject={(e) => wsActions.handleUpdateProject(e)}
  on:deleteProject={(e) => wsActions.handleDeleteProject(e)}
/>

<CommandPalette
  open={$modals.commandPalette}
  tasks={$allTasksIncludingArchived}
  sprints={$sprints}
  projects={$projectList}
  assignees={$wsAssignees}
  t={(key, options) => $_(key, options)}
  toggleTheme={() => theme.toggle()}
  switchView={(v) => {
    if (v === "test-cases") return;
    goto(`${base}/workspace/${workspaceId}${$page.url.search}`);
  }}
  openTask={(t) => {
    void openTaskPage(t);
  }}
  createTask={() => {
    $wsEditingTask = null;
    ui.openModal("form");
  }}
  startTimer={() => {}}
  openQuickNotes={() => {}}
  applyGlobalSearch={(q) => {
    searchInput.set(q);
    ui.closeModal("commandPalette");
    goto(`${base}/workspace/${workspaceId}${$page.url.search}`);
  }}
  openBookmarks={() => {}}
  openWhiteboard={() => {}}
  on:close={() => ui.closeModal("commandPalette")}
  dailyReflect={() => ui.openModal("dailyReflect")}
  openPageSize={() => ui.openModal("pageSize")}
/>

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

  :global(.property-select[data-status="draft"] .property-trigger-btn) {
    border-color: rgba(148, 163, 184, 0.35) !important;
  }
  :global(.property-select[data-status="draft"] .property-trigger-btn span),
  :global(.property-select[data-status="draft"] .property-trigger-btn svg) {
    color: #94a3b8 !important;
  }
  :global(.property-select[data-status="actual"] .property-trigger-btn) {
    border-color: rgba(96, 165, 250, 0.5) !important;
  }
  :global(.property-select[data-status="actual"] .property-trigger-btn span),
  :global(.property-select[data-status="actual"] .property-trigger-btn svg) {
    color: #60a5fa !important;
  }
  :global(.property-select[data-status="passed"] .property-trigger-btn) {
    border-color: rgba(52, 211, 153, 0.5) !important;
  }
  :global(.property-select[data-status="passed"] .property-trigger-btn span),
  :global(.property-select[data-status="passed"] .property-trigger-btn svg) {
    color: #34d399 !important;
  }
  :global(.property-select[data-status="failed"] .property-trigger-btn) {
    border-color: rgba(248, 113, 113, 0.5) !important;
  }
  :global(.property-select[data-status="failed"] .property-trigger-btn span),
  :global(.property-select[data-status="failed"] .property-trigger-btn svg) {
    color: #f87171 !important;
  }
  :global(.property-select[data-status="blocked"] .property-trigger-btn) {
    border-color: rgba(251, 146, 60, 0.5) !important;
  }
  :global(.property-select[data-status="blocked"] .property-trigger-btn span),
  :global(.property-select[data-status="blocked"] .property-trigger-btn svg) {
    color: #fb923c !important;
  }
  :global(.property-select[data-status="deprecated"] .property-trigger-btn) {
    border-color: rgba(100, 116, 139, 0.3) !important;
    opacity: 0.6;
  }
  :global(.property-select[data-status="deprecated"] .property-trigger-btn span),
  :global(.property-select[data-status="deprecated"] .property-trigger-btn svg) {
    color: #64748b !important;
  }
</style>
