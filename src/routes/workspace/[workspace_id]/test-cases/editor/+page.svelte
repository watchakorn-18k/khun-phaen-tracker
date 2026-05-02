<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { currentWorkspaceName } from "$lib/stores/workspace";
  import { getAssignees } from "$lib/db";
  import { api } from "$lib/apis";
  import { user } from "$lib/stores/auth";
  import { createUIActions } from "$lib/stores/uiActions";
  import type { Assignee } from "$lib/types";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import {
    ArrowLeft,
    CalendarDays,
    CheckCircle2,
    ClipboardCheck,
    Hash,
    Keyboard,
    ListChecks,
    MessageSquare,
    Paperclip,
    Plus,
    X,
    ShieldCheck,
    Target,
    Trash2,
    UserCheck,
    UserCog,
    Wrench,
    CircleDashed,
    XCircle,
    Ban,
    History,
    Check,
    AlertCircle,
    Shield,
    Folder,
    Layout,
    Clock,
    FileText,
  } from "lucide-svelte";
  import TestCaseStepList from "$lib/components/TestCaseStepList.svelte";

  type Status = "draft" | "actual" | "failed" | "blocked" | "deprecated" | "pass";
  type StepFormat = "gherkin" | "classic";
  type GherkinKeyword = "given" | "when" | "then" | "and";
  type GherkinStep = { keyword: GherkinKeyword; text: string };
  type ClassicStep = { action: string; data: string; expected: string };

  let suiteOptions: any[] = [];
  $: isAuthorized = $user?.role === "admin" || $user?.profile?.position === "QA Tester";
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

  const statusOptions = [
    { value: "draft", label: "Draft", icon: CircleDashed, iconClass: "text-gray-400" },
    { value: "actual", label: "Actual", icon: ListChecks, iconClass: "text-blue-400" },
    { value: "pass", label: "Pass", icon: CheckCircle2, iconClass: "text-green-400" },
    { value: "failed", label: "Failed", icon: XCircle, iconClass: "text-rose-400" },
    { value: "blocked", label: "Blocked", icon: Ban, iconClass: "text-gray-500" },
    { value: "deprecated", label: "Deprecated", icon: History, iconClass: "text-gray-600" },
  ];

  const priorityOptions = [
    { value: "high", label: "High", icon: Shield, iconClass: "text-rose-500" },
    { value: "medium", label: "Medium", icon: Shield, iconClass: "text-amber-500" },
    { value: "low", label: "Low", icon: Shield, iconClass: "text-gray-400" },
  ];

  let workspaceAssignees: Assignee[] = [];
  $: assigneeOptions = [
    { value: "unassigned", label: "Unassigned", user_id: undefined, avatarColor: "#64748b" },
    ...workspaceAssignees.map((a) => ({ 
      value: String(a.id || ""), 
      label: a.name,
      user_id: a.user_id,
      avatarUrl: a.avatar_url,
      avatarColor: a.color
    })),
  ];

  const fixedOptions = [
    { value: "no", label: "Not fixed", icon: AlertCircle, iconClass: "text-amber-400" },
    { value: "yes", label: "Fixed", icon: Check, iconClass: "text-green-400" },
  ];

  const stepFormatOptions = [
    { value: "gherkin", label: "Gherkin" },
    { value: "classic", label: "Classic" },
  ];


  const mockCases: Record<string, {
    name: string;
    suiteId: string;
    description: string;
    preconditions: string;
    postconditions: string;
    status: Status;
    assignee: string;
  }> = {
    "TC-1": {
      name: "Authorization",
      suiteId: "authorization",
      description: "Registered users can log in to the application using email and password.",
      preconditions: "User account exists and email is verified.",
      postconditions: "User lands on the workspace dashboard.",
      status: "actual",
      assignee: "qa-lead",
    },
    "TC-4": {
      name: "Sign up with invite link",
      suiteId: "authorization",
      description: "Invited users can join a workspace through a valid invite.",
      preconditions: "",
      postconditions: "",
      status: "draft",
      assignee: "wk-18k",
    },
  };

  $: workspaceId = $page.params.workspace_id;
  $: workspaceLabel = $currentWorkspaceName || "Current workspace";
  $: caseId = $page.url.searchParams.get("case") || "";
  $: isEditing = !!caseId;

  const initialCase = mockCases[caseId] || null;
  let name = initialCase?.name || "";
  let suiteId = initialCase?.suiteId || $page.url.searchParams.get("suite") || "";
  let description = initialCase?.description || "";
  let preconditions = initialCase?.preconditions || "";
  let postconditions = initialCase?.postconditions || "";
  let status: Status = initialCase?.status || "actual";
  let priority: "high" | "medium" | "low" = (initialCase as any)?.priority || "medium";
  let assignee = initialCase?.assignee || "unassigned";
  let testNo = caseId || "";
  let dateCreate = "2026-05-03";
  let input = "";
  let expectedResult = "";
  let actualResult = "";
  let fixed: "no" | "yes" = "no";
  let assignDev = "unassigned";
  let assignTester = initialCase?.assignee || "unassigned";
  let devNote = "";
  let testNote = "";
  let stepFormat: StepFormat = "classic";
  let showCreateSuiteModal = false;
  let newSuiteName = "";
  let isSaving = false;
  const ui = createUIActions();

  async function handleCreateSuite() {
    if (!newSuiteName.trim() || !workspaceId) return;
    
    try {
      const resp = await api.data.testSuites.create(workspaceId, { title: newSuiteName });
      if (resp.ok) {
        const newSuite = await resp.json();
        const newId = newSuite.id || newSuite._id;
        suiteOptions = [...suiteOptions, { 
          value: newId, 
          label: newSuite.title, 
          icon: Folder, 
          iconClass: "text-gray-400" 
        }];
        suiteId = newId;
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
  let gherkinSteps: GherkinStep[] = [
    { keyword: "given", text: "" },
    { keyword: "when", text: "" },
    { keyword: "then", text: "" },
  ];
  let gherkinView: "raw" | "steps" = "steps";
  let gherkinRaw = "";

  type AttachmentFile = { file: File; url: string };
  let attachments: AttachmentFile[] = [];
  let attachmentInput: HTMLInputElement;
  const MAX_ATTACHMENTS = 5;

  function addAttachments(event: Event) {
    const target = event.target as HTMLInputElement;
    if (!target.files) return;
    const newFiles = Array.from(target.files);
    const remaining = MAX_ATTACHMENTS - attachments.length;
    const toAdd = newFiles.slice(0, remaining);
    const created = toAdd.map((f) => ({ file: f, url: URL.createObjectURL(f) }));
    attachments = [...attachments, ...created];
    target.value = "";
  }

  function removeAttachment(index: number) {
    const removed = attachments[index];
    URL.revokeObjectURL(removed.url);
    attachments = attachments.filter((_, i) => i !== index);
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  onDestroy(() => {
    attachments.forEach((a) => URL.revokeObjectURL(a.url));
  });

  onMount(async () => {
    try {
      workspaceAssignees = await getAssignees(true);
      
      // Fetch suites
      if (workspaceId) {
        const suiteResp = await api.data.testSuites.list(workspaceId);
        if (suiteResp.ok) {
          const suites = await suiteResp.json();
          suiteOptions = suites.map((s: any) => ({
            value: s.id || s._id,
            label: s.title,
            icon: Folder,
            iconClass: "text-gray-400"
          }));
          
          // If suiteId is not in options (e.g. from URL), handle it
          if (suiteId && !suiteOptions.find(o => o.value === suiteId)) {
            // Might need to fetch the specific suite if it's not in the list, 
            // but usually list returns all for the workspace.
          } else if (!suiteId && suiteOptions.length > 0 && suiteOptions[0].value) {
            suiteId = suiteOptions[0].value;
          }
        }
      }

      // Fetch next test number
      if (!isEditing && workspaceId) {
        const resp = await api.data.testCases.nextNumber(workspaceId);
        if (resp.ok) {
          const data = await resp.json();
          testNo = `TC-${data.next_number}`;
        }
      }

      // Fetch test case if editing
      if (isEditing && caseId) {
        const tcResp = await api.data.testCases.get(caseId);
        if (tcResp.ok) {
          const tc = await tcResp.json();
          name = tc.name || "";
          suiteId = tc.suite_id || "";
          description = tc.description || "";
          preconditions = tc.preconditions || "";
          postconditions = tc.postconditions || "";
          status = tc.status || "actual";
          priority = tc.priority || "medium";
          assignDev = tc.assign_dev || "unassigned";
          assignTester = tc.assign_tester || "unassigned";
          testNo = `TC-${tc.test_no}`;
          input = tc.input || "";
          expectedResult = tc.expected_result || "";
          actualResult = tc.actual_result || "";
          fixed = tc.fixed || "no";
          devNote = tc.dev_note || "";
          testNote = tc.test_note || "";
          stepFormat = tc.step_format || "classic";
          
          if (tc.step_format === "gherkin") {
            gherkinSteps = tc.gherkin_steps && tc.gherkin_steps.length > 0 ? tc.gherkin_steps : [
              { keyword: "given", text: "" },
              { keyword: "when", text: "" },
              { keyword: "then", text: "" },
            ];
          } else if (tc.classic_steps && tc.classic_steps.length > 0) {
            classicSteps = tc.classic_steps;
          }
        }
      }
    } catch (e) {
      console.error("Failed to initialize editor:", e);
    }
  });

  let classicSteps: ClassicStep[] = isEditing
    ? [
        {
          action: "When Go to sign up page https://qase.io/login",
          data: "Data",
          expected: "Then Sign up page is shown",
        },
        {
          action: "When Fill the form with login \"test\" and password \"test\"",
          data: "Data",
          expected: "Then Form is being submitted and user is redirected to https://qase.io/app page.",
        },
      ]
    : [{ action: "", data: "", expected: "" }];

  function backToRepository() {
    const params = new URLSearchParams($page.url.searchParams);
    params.delete("suite");
    params.delete("case");
    const query = params.toString();
    goto(`${base}/workspace/${workspaceId}/test-cases${query ? `?${query}` : ""}`);
  }


  async function handleSubmit() {
    if (!name.trim()) return;
    isSaving = true;

    try {
      const payload: any = {
        suite_id: (suiteId && suiteId !== "undefined") ? String(suiteId) : null,
        name: name.trim(),
        description,
        preconditions,
        postconditions,
        input,
        expected_result: expectedResult,
        actual_result: actualResult,
        status,
        priority,
        fixed,
        assign_dev: assignDev,
        assign_tester: assignTester,
        dev_note: devNote,
        test_note: testNote,
        step_format: stepFormat,
        classic_steps: stepFormat === "classic" ? classicSteps : undefined,
        gherkin_steps: stepFormat === "gherkin" ? gherkinSteps : undefined,
      };

      if (!workspaceId) return;
      
      const resp = isEditing && caseId
        ? await api.data.testCases.update(caseId, payload)
        : await api.data.testCases.create(workspaceId, payload);
      
      if (resp.ok) {
        const result = isEditing ? { id: caseId } : await resp.json();
        const testCaseId = result.id;

        // Upload attachments if any
        if (attachments.length > 0) {
          const formData = new FormData();
          attachments.forEach((a) => {
            formData.append("files", a.file);
          });

          await api.data.testCases.uploadAttachment(workspaceId, testCaseId, formData);
        }

        ui.showMessage(isEditing ? "Test case updated successfully" : "Test case created successfully", "success");
        backToRepository();
      } else {
        const error = await resp.json();
        ui.showMessage(error.error || `Failed to ${isEditing ? 'update' : 'create'} test case`, "error");
      }
    } catch (e) {
      console.error("Save error:", e);
      ui.showMessage("An unexpected error occurred", "error");
    } finally {
      isSaving = false;
    }
  }

  function autoGrow(node: HTMLTextAreaElement) {
    const resize = () => {
      node.style.height = "auto";
      node.style.height = `${node.scrollHeight}px`;
    };

    resize();
    node.addEventListener("input", resize);

    return {
      update: resize,
      destroy() {
        node.removeEventListener("input", resize);
      },
    };
  }
</script>

<svelte:head>
  <title>{isEditing ? "Edit" : "Create"} Test Case - {workspaceLabel}</title>
</svelte:head>

<div class="min-h-screen bg-[#0b1220] text-white">
  <form class="mx-auto flex min-h-screen w-full max-w-6xl flex-col px-6 py-5" on:submit|preventDefault={handleSubmit}>
    <header class="flex items-center justify-between gap-4">
      <button
        type="button"
        class="inline-flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm font-bold text-gray-400 hover:bg-white/5 hover:text-white"
        on:click={backToRepository}
      >
        <ArrowLeft size={16} />
        Back
      </button>
      <button
        type="submit"
        class="flex h-10 items-center gap-2 rounded-xl bg-indigo-600 px-6 text-sm font-black text-white shadow-lg shadow-indigo-600/25 hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
        disabled={isSaving || !name.trim()}
      >
        {#if isSaving}
          <Clock size={16} class="animate-spin" />
          <span>Saving...</span>
        {:else}
          {isEditing ? "Save test case" : "Create test case"}
        {/if}
      </button>
    </header>

    <main class="mt-6 flex-1 rounded-2xl border border-white/10 bg-[#111827] shadow-2xl shadow-black/30">
      <div class="border-b border-white/5 px-6 py-5">
        <div class="flex min-w-0 items-center gap-2 text-[13px] font-medium text-gray-500">
          <span class="truncate">{workspaceLabel}</span>
          <span class="text-gray-600">›</span>
          <span class="text-gray-400">{isEditing ? caseId : "Create test case"}</span>
        </div>

        <div class="mt-8 flex flex-wrap items-center gap-3 text-[12px] font-black uppercase tracking-widest text-gray-500">
          <span class="inline-flex items-center gap-2 rounded-full border border-white/10 px-3 py-1.5">
            <Hash size={14} />
            {testNo || "Auto test no"}
          </span>
          <span class="inline-flex items-center gap-2 rounded-full border border-white/10 px-3 py-1.5">
            <CalendarDays size={14} />
            {dateCreate}
          </span>
        </div>

        <input
          bind:value={name}
          readonly={!isAuthorized}
          class="mt-4 w-full !bg-transparent px-0 py-2 text-2xl font-black text-white placeholder:text-gray-600 border-none outline-none {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
          placeholder="Example: Validate invite-link signup flow..."
        />

        <label class="mt-6 block">
          <textarea
            bind:value={description}
            readonly={!isAuthorized}
            use:autoGrow
            class="min-h-[38px] w-full overflow-hidden resize-none !bg-transparent px-0 text-[15px] leading-relaxed text-gray-300 placeholder:text-gray-600 border-none outline-none {isAuthorized ? '' : 'cursor-default'}"
            placeholder="Additional details..."
          ></textarea>
        </label>

        <div class="mt-6 grid grid-cols-1 gap-4 md:grid-cols-2">
          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <UserCog size={14} />
              Assign Dev
            </span>
            <div class="property-select">
              <SearchableSelect id="editor-assign-dev" bind:value={assignDev} options={availableAssigneeOptions} minimal={true} />
            </div>
          </label>

          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <UserCheck size={14} />
              Assign Tester
            </span>
            <div class="property-select">
              {#if isAuthorized}
                <SearchableSelect id="editor-assign-tester" bind:value={assignTester} options={assigneeOptions} minimal={true} />
              {:else}
                <div class="pointer-events-none opacity-80">
                  <SearchableSelect id="editor-assign-tester" bind:value={assignTester} options={assigneeOptions} minimal={true} />
                </div>
              {/if}
            </div>
          </label>
        </div>

        <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-4">
          {#if isAuthorized}
            <label class="space-y-2">
              <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
                <ClipboardCheck size={14} />
                Status
              </span>
              <div class="property-select">
                <SearchableSelect id="editor-status" bind:value={status} options={statusOptions} showSearch={false} minimal={true} />
              </div>
            </label>
          {:else}
            <label class="space-y-2 opacity-80 pointer-events-none">
              <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
                <ClipboardCheck size={14} />
                Status
              </span>
              <div class="property-select">
                <SearchableSelect id="editor-status" bind:value={status} options={statusOptions} showSearch={false} minimal={true} />
              </div>
            </label>
          {/if}

          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <Shield size={14} />
              Priority
            </span>
            <div class="property-select">
              {#if isAuthorized}
                <SearchableSelect id="editor-priority" bind:value={priority} options={priorityOptions} showSearch={false} minimal={true} />
              {:else}
                <div class="pointer-events-none opacity-80">
                  <SearchableSelect id="editor-priority" bind:value={priority} options={priorityOptions} showSearch={false} minimal={true} />
                </div>
              {/if}
            </div>
          </label>

          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <Wrench size={14} />
              Fixed
            </span>
            <div class="property-select">
              <SearchableSelect id="editor-fixed" bind:value={fixed} options={fixedOptions} showSearch={false} minimal={true} />
            </div>
          </label>

          <label class="space-y-2">
            <span class="flex items-center justify-between">
              <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
                <ListChecks size={14} />
                Suite
              </span>
              {#if isAuthorized}
                <button
                  type="button"
                  on:click={() => (showCreateSuiteModal = true)}
                  class="flex h-5 w-5 items-center justify-center rounded-md text-gray-500 hover:bg-white/10 hover:text-white"
                  title="Create new suite"
                >
                  <Plus size={12} />
                </button>
              {/if}
            </span>
            <div class="property-select">
              {#if isAuthorized}
                <SearchableSelect id="editor-suite" bind:value={suiteId} options={suiteOptions} minimal={true} />
              {:else}
                <div class="pointer-events-none opacity-80">
                  <SearchableSelect id="editor-suite" bind:value={suiteId} options={suiteOptions} minimal={true} />
                </div>
              {/if}
            </div>
          </label>
        </div>
      </div>

      <section class="grid grid-cols-1 gap-5 border-b border-white/5 px-6 py-6 md:grid-cols-2">
        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <ShieldCheck size={14} />
            Precondition
          </span>
          <textarea
            bind:value={preconditions}
            readonly={!isAuthorized}
            use:autoGrow
            rows="1"
            class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
            placeholder="Conditions required before this test starts..."
          ></textarea>
        </label>
        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <Target size={14} />
            Post-conditions
          </span>
          <textarea
            bind:value={postconditions}
            readonly={!isAuthorized}
            use:autoGrow
            rows="1"
            class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
            placeholder="Expected state after this test finishes..."
          ></textarea>
        </label>
      </section>

      <section class="border-b border-white/5 px-6 py-6">
        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <Keyboard size={14} />
            Input
          </span>
          <textarea
            bind:value={input}
            readonly={!isAuthorized}
            use:autoGrow
            rows="1"
            class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
            placeholder="Data that tester must enter..."
          ></textarea>
        </label>

        <div class="mt-5 grid grid-cols-1 gap-5 lg:grid-cols-2">
          <label>
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <CheckCircle2 size={14} />
              Expected Result
            </span>
            <textarea
              bind:value={expectedResult}
              readonly={!isAuthorized}
              use:autoGrow
              rows="1"
              class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
              placeholder="Expected behavior after running the step..."
            ></textarea>
          </label>

          <label>
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <ClipboardCheck size={14} />
              Actual Result
            </span>
            <textarea
              bind:value={actualResult}
              readonly={!isAuthorized}
              use:autoGrow
              rows="1"
              class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
              placeholder="Actual behavior found during testing..."
            ></textarea>
          </label>
        </div>
      </section>

      <section class="grid grid-cols-1 gap-5 border-b border-white/5 px-6 py-6 lg:grid-cols-2">

        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <MessageSquare size={14} />
            Dev Note
          </span>
          <textarea
            bind:value={devNote}
            use:autoGrow
            rows="1"
            class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
            placeholder="Developer note..."
          ></textarea>
        </label>

        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <MessageSquare size={14} />
            Test Note
          </span>
          <textarea
            bind:value={testNote}
            readonly={!isAuthorized}
            use:autoGrow
            rows="1"
            class="mt-3 min-h-[38px] w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20 {isAuthorized ? '' : 'cursor-default focus:ring-0'}"
            placeholder="Tester note..."
          ></textarea>
        </label>
      </section>

      <section class="border-b border-white/5 px-6 py-6">
        <h2 class="text-base font-black text-gray-100">Attachments</h2>
        <div class="mt-1 border-t border-white/10"></div>

        {#if attachments.length > 0}
          <div class="mt-4 flex flex-wrap gap-3">
            {#each attachments as att, index}
              <div class="group relative w-40 overflow-hidden rounded-lg border border-white/10 bg-[#1a2332]">
                {#if att.file.type.startsWith('image/')}
                  <img src={att.url} alt={att.file.name} class="h-24 w-full object-cover" />
                {:else}
                  <div class="flex h-24 items-center justify-center bg-[#151e2e]">
                    <Paperclip size={28} class="text-gray-500" />
                  </div>
                {/if}
                <div class="px-2 py-1.5">
                  <p class="truncate text-xs font-medium text-gray-300">{att.file.name}</p>
                  <div class="flex items-center justify-between">
                    <span class="text-[11px] text-gray-500">{formatFileSize(att.file.size)}</span>
                      {#if isAuthorized}
                        <button
                          type="button"
                          class="grid h-5 w-5 place-items-center rounded text-gray-500 hover:text-rose-400 transition-colors"
                          title="Remove"
                          on:click={() => removeAttachment(index)}
                        >
                          <X size={13} />
                        </button>
                      {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if isAuthorized && attachments.length < MAX_ATTACHMENTS}
          <input
            bind:this={attachmentInput}
            type="file"
            multiple
            accept="image/*,.pdf,.doc,.docx,.txt,.csv,.xlsx"
            class="hidden"
            on:change={addAttachments}
          />
          <button
            type="button"
            class="mt-4 inline-flex items-center gap-1.5 text-sm font-black text-indigo-400 hover:text-indigo-300"
            on:click={() => attachmentInput.click()}
          >
            <Plus size={15} />
            Add attachment
          </button>
          <span class="ml-2 text-xs text-gray-600">({attachments.length}/{MAX_ATTACHMENTS})</span>
        {:else if isAuthorized}
          <p class="mt-4 text-xs text-gray-500">Maximum {MAX_ATTACHMENTS} attachments reached.</p>
        {/if}
      </section>

      <section class="px-6 py-6">
        <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <h2 class="text-base font-black text-gray-100">Test Case Steps</h2>
            <div class="property-select min-w-[104px]">
              <SearchableSelect
                id="editor-step-format"
                bind:value={stepFormat}
                options={stepFormatOptions}
                showSearch={false}
                minimal={true}
              />
            </div>
          </div>
        </div>

        {#if stepFormat === "classic"}
          <TestCaseStepList
            bind:steps={classicSteps}
            format="classic"
            readOnly={!isAuthorized}
          />
        {:else}
          <TestCaseStepList
            bind:steps={gherkinSteps}
            format="gherkin"
            readOnly={!isAuthorized}
            bind:gherkinView
            bind:gherkinRaw
          />
        {/if}
      </section>
    </main>
  </form>
</div>

{#if showCreateSuiteModal}
  <div class="fixed inset-0 z-[9999] flex items-center justify-center p-4" transition:fade={{ duration: 150 }}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" on:click={() => (showCreateSuiteModal = false)}></div>
    <div class="relative w-full max-w-md rounded-2xl border border-white/10 bg-[#111827] p-6 shadow-2xl">
      <h3 class="text-lg font-black text-white">Create New Suite</h3>
      <p class="mt-1 text-sm text-gray-500">Enter a name for the new test suite.</p>

      <div class="mt-6">
        <label for="new-suite-name" class="block text-[12px] font-black uppercase tracking-widest text-gray-500">
          Suite Name
        </label>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="new-suite-name"
          type="text"
          bind:value={newSuiteName}
          class="mt-3 w-full rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-white outline-none focus:border-white/20"
          placeholder="e.g. Authentication, Dashboard..."
          autofocus
          on:keydown={(e) => e.key === "Enter" && handleCreateSuite()}
        />
      </div>

      <div class="mt-8 flex items-center justify-end gap-3">
        <button
          type="button"
          class="rounded-xl px-4 py-2 text-sm font-bold text-gray-400 hover:bg-white/5 hover:text-white"
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

<style>
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
