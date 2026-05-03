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
    status: "draft" | "actual" | "failed" | "blocked" | "deprecated" | "pass";
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
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.filter((c) => c.id !== caseToDeleteId),
        }));
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
    try {
      const resp = await api.data.testCases.updatePriority(
        testCase.id,
        newPriority,
      );
      if (resp.ok) {
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, priority: newPriority as any } : c,
          ),
        }));
        ui.showMessage("Priority updated", "success");
      } else {
        ui.showMessage("Failed to update priority", "error");
      }
    } catch (e) {
      ui.showMessage("Failed to update priority", "error");
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
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, assignee: newAssignTester } : c,
          ),
        }));
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
        suites = suites.map((s) => ({
          ...s,
          cases: s.cases.map((c) =>
            c.id === testCase.id ? { ...c, assign_dev: newAssignDev } : c,
          ),
        }));
        ui.showMessage("Assignee updated", "success");
      } else {
        const err = await resp.text();
        ui.showMessage(`Failed to update assignee: ${err}`, "error");
      }
    } catch (e) {
      ui.showMessage("Failed to update assignee", "error");
    }
  }

  async function loadData(q?: string, searchField?: string) {
    if (!workspaceId) return;
    try {
      const priorityQuery =
        activeFilters.priority?.length > 0
          ? activeFilters.priority.join(",")
          : undefined;
      const statusQuery =
        activeFilters.status?.length > 0
          ? activeFilters.status.join(",")
          : undefined;
      const fixedQuery =
        activeFilters.fixed?.length > 0
          ? activeFilters.fixed.join(",")
          : undefined;

      const [suiteResp, caseResp] = await Promise.all([
        api.data.testSuites.list(workspaceId),
        api.data.testCases.list(workspaceId, {
          q,
          field: searchField,
          priority: priorityQuery,
          status: statusQuery,
          fixed: fixedQuery,
          assign_dev: activeFilters.assign_dev?.length > 0
            ? activeFilters.assign_dev.join(",")
            : undefined,
        }),
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

  let searchTimeout: any;
  $: if ((query !== undefined || field !== undefined) && browser) {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      loadData(query, field);
    }, 300);
  }

  $: monthlySummary = buildMonthlySummary($allTasksIncludingArchived);

  onMount(async () => {
    document.addEventListener("keydown", keyboardHandler);
    assignees = await getAssignees(true);
    await loadData();
    ws.loadData();
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

  function exportToCSV() {
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
    let rows: string[][] = [];

    suites.forEach((suite) => {
      suite.cases.forEach((c) => {
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

        rows.push([
          suite.title,
          String(c.test_no),
          c.title,
          c.status,
          c.assignee || "unassigned",
          c.assign_dev || "unassigned",
          c.preconditions || "",
          c.input || "",
          c.expected_result || "",
          c.actual_result || "",
          steps,
        ]);
      });
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
  }

  function parseCSVLine(line: string): string[] {
    const result = [];
    let current = "";
    let inQuotes = false;
    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      if (char === '"') {
        if (inQuotes && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (char === "," && !inQuotes) {
        result.push(current);
        current = "";
      } else {
        current += char;
      }
    }
    result.push(current);
    return result;
  }

  async function handleImport(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    const reader = new FileReader();

    reader.onload = async (e) => {
      const text = e.target?.result as string;
      const lines = text.split(/\r?\n/);
      if (lines.length <= 1) return;

      const headers = lines[0]
        .split(",")
        .map((h) => h.replace(/^"|"$/g, "").trim());

      const casesToCreate = [];
      for (let i = 1; i < lines.length; i++) {
        if (!lines[i].trim()) continue;

        const values = parseCSVLine(lines[i]);
        const row: Record<string, string> = {};
        headers.forEach((h, idx) => {
          row[h] = (values[idx] || "").trim();
        });

        const title = row["Test Name"] || row["title"];
        if (!title) continue;

        const stepsStr = row["Test Step"] || row["steps"] || "";
        const classicSteps = stepsStr
          .split("\n")
          .map((s) => ({
            action: s.trim(),
            data: "",
            expected: "",
          }))
          .filter((s) => s.action);

        casesToCreate.push({
          name: title,
          description: "",
          preconditions: row["Precondition"] || "",
          postconditions: "",
          input: row["Input"] || "",
          expected_result: row["Expected Result"] || "",
          actual_result: row["Actual Result"] || "",
          status: row["Status"] || "actual",
          priority: "medium",
          fixed: "no",
          assign_dev: row["Assign Dev"] || "unassigned",
          assign_tester: row["Assign Tester"] || "unassigned",
          step_format: "classic",
          classic_steps: classicSteps.length > 0 ? classicSteps : undefined,
          suite_id: selectedSuiteId === "unassigned" ? null : selectedSuiteId,
        });
      }

      if (casesToCreate.length > 0) {
        let successCount = 0;
        ui.showMessage(`Importing ${casesToCreate.length} cases...`, "success");
        for (const tc of casesToCreate) {
          try {
            if (!workspaceId) continue;
            const resp = await api.data.testCases.create(workspaceId, tc);
            if (resp.ok) successCount++;
          } catch (err) {
            console.error("Import error:", err);
          }
        }
        ui.showMessage(`Imported ${successCount} test cases`, "success");
        // Refetch suites
        if (workspaceId) {
          const suiteResp = await api.data.testSuites.list(workspaceId);
          const caseResp = await api.data.testCases.list(workspaceId);
          if (suiteResp.ok && caseResp.ok) {
            const suiteData = await suiteResp.json();
            const caseData = await caseResp.json();
            const mappedCases = caseData.map(mapBackendToFrontend);
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
                mappedCases.filter((c: TestCase) => c.suite_id === s.id)
                  .length >= 10,
            }));
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
          }
        }
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
    { value: "high", label: "High" },
    { value: "medium", label: "Medium" },
    { value: "low", label: "Low" },
  ];

  const statusOptions = [
    {
      value: "draft",
      label: "Draft",
      icon: CircleDashed,
      iconClass: "text-slate-400",
    },
    {
      value: "actual",
      label: "Actual",
      icon: ListChecks,
      iconClass: "text-blue-400",
    },
    {
      value: "pass",
      label: "Pass",
      icon: CheckCircle2,
      iconClass: "text-green-400",
    },
    {
      value: "failed",
      label: "Failed",
      icon: XCircle,
      iconClass: "text-rose-400",
    },
    {
      value: "blocked",
      label: "Blocked",
      icon: Ban,
      iconClass: "text-slate-500",
    },
    {
      value: "deprecated",
      label: "Deprecated",
      icon: History,
      iconClass: "text-slate-600",
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
        <div class="flex flex-col gap-4">
          <div class="flex items-baseline gap-3">
            <h1
              class="text-2xl font-black tracking-tight text-slate-800 dark:text-white"
            >
              {workspaceLabel} repository
            </h1>
            <span class="text-sm font-medium text-slate-500">
              {caseCount} cases ({caseCount}) | {suiteCount} suites ({suiteCount})
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
                    placeholder="Search test cases, ID, description..."
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
                    <span>{values.map((v) => getFilterLabel(prop, v)).join(", ")}</span>
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
                        <div class="h-px bg-slate-100 dark:bg-gray-700 my-1"></div>
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
                      <div class="flex flex-col py-1 max-h-[400px] overflow-y-auto">
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
                        <div class="h-px bg-slate-100 dark:bg-gray-700 mb-1"></div>

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
                                checked={activeFilters.status.includes(opt.value)}
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
                                checked={activeFilters.fixed.includes(opt.value)}
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
                  title="Actions"
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
            {#if isAuthorized}
              <button
                class="grid h-8 w-8 place-items-center rounded-lg text-slate-500 hover:bg-slate-100 dark:text-gray-400 dark:hover:bg-gray-800"
                title="Add suite"
                on:click={() => (showCreateSuiteModal = true)}
              >
                <Plus size={16} />
              </button>
            {/if}
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
                  {#if isAuthorized}
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
                  {/if}
                </div>

                <div class="divide-y divide-slate-200 dark:divide-gray-800">
                  {#each suite.cases || [] as testCase}
                    <div
                      class="grid w-full grid-cols-[24px_52px_1fr_auto_auto_auto] sm:grid-cols-[60px_52px_1fr_auto_auto_auto] items-center gap-3 rounded-md px-3 py-2 text-left transition-colors {selectedCaseId ===
                      testCase.id
                        ? 'bg-slate-100 dark:bg-transparent'
                        : 'hover:bg-slate-50 dark:hover:bg-gray-900/70'}"
                    >
                      <button
                        class="col-span-3 grid grid-cols-[24px_52px_1fr] sm:grid-cols-[60px_52px_1fr] items-center gap-3 text-left"
                        on:click={() => {
                          selectCase(suite, testCase);
                          showDetail = true;
                        }}
                      >
                        <span class="grid h-5 w-5 place-items-center sm:hidden">
                          <span
                            class="h-2.5 w-2.5 rounded-full border-2 {testCase.priority ===
                            'high'
                              ? 'border-rose-500'
                              : 'border-slate-300'}"
                          ></span>
                        </span>
                        <span
                          class="hidden w-full items-center justify-center rounded-md border px-1 py-0.5 text-[10px] font-black uppercase tracking-wide sm:flex {priorityColor(
                            testCase.priority,
                          )}"
                        >
                          {testCase.priority}
                        </span>
                        <span
                          class="font-mono text-sm font-bold text-slate-400 text-left"
                          >TC-{testCase.test_no}</span
                        >
                        <span
                          class="min-w-0 truncate text-sm font-bold text-slate-700 dark:text-gray-200 text-left"
                          >{testCase.title}</span
                        >
                      </button>
                      <div class="flex items-center gap-2">
                        {#if isAuthorized}
                          <div class="property-select min-w-[80px]">
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
                          title={isAuthorized ? "Edit Case" : "View Case"}
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
                            title="Delete Case"
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
                    {:else}
                      <div class="flex-1"></div>
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
              title="Close details"
              on:click={() => (showDetail = false)}
            >
              <X size={18} />
            </button>
          </div>

          <div class="mt-4 flex flex-wrap items-center gap-2">
            {#if isAuthorized}
              <button
                class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-500 hover:bg-slate-200 transition-colors dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                title="Edit"
                on:click={() =>
                  openTestCaseEditor(selectedSuite.id, selectedCase.id)}
              >
                <Edit3 size={17} />
              </button>
              <button
                class="grid h-9 w-9 place-items-center rounded-lg bg-slate-100 text-slate-500 hover:bg-slate-200 transition-colors dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                title="Delete"
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
                <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
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
                      on:click={() => startEditing("input", selectedCase.input || "")}
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
                  <p class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap">
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
                  <p class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap">
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
                  <p class="mt-2 text-sm font-medium text-slate-500 whitespace-pre-wrap">
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
                placeholder="Add dev notes..."
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
                <h3 class="text-sm font-black text-slate-700 dark:text-gray-200">
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
        title="Zoom in"><ZoomIn size={18} /></button
      >
      <button
        type="button"
        on:click={lbZoomOut}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title="Zoom out"><ZoomOut size={18} /></button
      >
      <button
        type="button"
        on:click={lbRotateRight}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title="Rotate"><RotateCw size={18} /></button
      >
      <button
        type="button"
        on:click={lbResetView}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title="Reset"><RotateCcw size={18} /></button
      >
      <div class="w-px h-5 bg-white/20"></div>
      <button
        type="button"
        on:click={() => void lbDownload()}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title="Download"><Download size={18} /></button
      >
      <button
        type="button"
        on:click={lbOpenInNewTab}
        class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors"
        title="Open in new tab"><ExternalLink size={18} /></button
      >
      <div class="w-px h-5 bg-white/20"></div>
      <button
        type="button"
        on:click={closeLightbox}
        class="p-2 bg-black/40 hover:bg-red-600/80 text-white rounded-lg transition-colors"
        title="Close"><X size={18} /></button
      >
    </div>

    {#if lightboxImages.length > 1}
      <button
        type="button"
        on:click={lbNavigatePrev}
        class="absolute left-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
        title="Previous"
      >
        <ChevronLeft size={24} />
      </button>
      <button
        type="button"
        on:click={lbNavigateNext}
        class="absolute right-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
        title="Next"
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
        lightboxZoom}px, {lightboxY / lightboxZoom}px) rotate({lightboxRotation}deg);"
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
  show={showDeleteModal}
  title="Delete Test Case"
  message="Are you sure you want to delete this test case? This action cannot be undone."
  confirmText="Delete"
  cancelText="Cancel"
  type="danger"
  onConfirm={confirmDeleteTestCase}
  onClose={() => {
    showDeleteModal = false;
    caseToDeleteId = "";
  }}
/>

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
    if (v === 'test-cases') return;
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
</style>
