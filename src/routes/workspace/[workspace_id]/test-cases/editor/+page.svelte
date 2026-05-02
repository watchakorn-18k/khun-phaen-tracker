<script lang="ts">
  import { onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { currentWorkspaceName } from "$lib/stores/workspace";
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
  } from "lucide-svelte";

  type Status = "draft" | "actual" | "failed" | "blocked" | "deprecated";
  type StepFormat = "gherkin" | "classic";
  type GherkinKeyword = "given" | "when" | "then" | "and";
  type GherkinStep = { keyword: GherkinKeyword; text: string };
  type ClassicStep = { action: string; data: string; expected: string };

  const suiteOptions = [
    { value: "authorization", label: "Authorization" },
    { value: "projects", label: "Projects" },
    { value: "workspace", label: "Workspace" },
  ];

  const statusOptions = [
    { value: "draft", label: "Draft" },
    { value: "actual", label: "Actual" },
    { value: "failed", label: "Failed" },
    { value: "blocked", label: "Blocked" },
    { value: "deprecated", label: "Deprecated" },
  ];

  const assigneeOptions = [
    { value: "unassigned", label: "Unassigned" },
    { value: "wk-18k", label: "WK 18K" },
    { value: "qa-lead", label: "QA Lead" },
    { value: "product-owner", label: "Product Owner" },
  ];

  const fixedOptions = [
    { value: "no", label: "Not fixed" },
    { value: "yes", label: "Fixed" },
  ];

  const stepFormatOptions = [
    { value: "gherkin", label: "Gherkin" },
    { value: "classic", label: "Classic" },
  ];

  const gherkinKeywordOptions = [
    { value: "given", label: "Given" },
    { value: "when", label: "When" },
    { value: "then", label: "Then" },
    { value: "and", label: "And" },
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
  let suiteId = initialCase?.suiteId || $page.url.searchParams.get("suite") || "authorization";
  let description = initialCase?.description || "";
  let preconditions = initialCase?.preconditions || "";
  let postconditions = initialCase?.postconditions || "";
  let status: Status = initialCase?.status || "actual";
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

  function addGherkinStep() {
    const next: GherkinKeyword = gherkinSteps.length % 3 === 0 ? "given" : gherkinSteps.length % 3 === 1 ? "when" : "then";
    gherkinSteps = [...gherkinSteps, { keyword: next, text: "" }];
  }

  function removeGherkinStep(index: number) {
    gherkinSteps = gherkinSteps.filter((_, stepIndex) => stepIndex !== index);
  }

  function convertToRaw() {
    gherkinRaw = gherkinSteps
      .filter((s) => s.text.trim() || s.keyword)
      .map((s) => `${s.keyword.charAt(0).toUpperCase() + s.keyword.slice(1)} ${s.text}`)
      .join("\n");
  }

  function convertToSteps() {
    const lines = gherkinRaw.split("\n").filter((line) => line.trim() !== "");
    const parsed = lines.map((line) => {
      const trimmed = line.trim();
      const firstSpace = trimmed.indexOf(" ");
      if (firstSpace !== -1) {
        const firstWord = trimmed.substring(0, firstSpace).toLowerCase();
        if (["given", "when", "then", "and"].includes(firstWord)) {
          return { keyword: firstWord as GherkinKeyword, text: trimmed.substring(firstSpace + 1) };
        }
      }
      return { keyword: "given" as GherkinKeyword, text: trimmed };
    });
    
    if (parsed.length > 0) {
      gherkinSteps = parsed;
    } else {
      gherkinSteps = [{ keyword: "given", text: "" }];
    }
  }

  function highlightGherkin(text: string) {
    if (!text) return "";
    return text
      .split("\n")
      .map((line) => {
        const match = line.match(/^(\s*)(Given|When|Then|And)\b(.*)/i);
        if (match) {
          const rest = match[3].replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
          return `${match[1]}<span class="text-sky-400 font-bold">${match[2]}</span><span class="text-gray-200">${rest}</span>`;
        }
        return `<span class="text-gray-200">${line.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")}</span>`;
      })
      .join("<br>");
  }

  function addClassicStep() {
    classicSteps = [...classicSteps, { action: "", data: "", expected: "" }];
  }

  function removeClassicStep(index: number) {
    classicSteps = classicSteps.filter((_, stepIndex) => stepIndex !== index);
  }

  function handleSubmit() {
    backToRepository();
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
        class="h-10 rounded-xl bg-indigo-600 px-6 text-sm font-black text-white shadow-lg shadow-indigo-600/25 hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-indigo-500/40"
        disabled={!name.trim()}
      >
        {isEditing ? "Save test case" : "Create test case"}
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
          class="mt-4 w-full !bg-transparent px-0 py-2 text-2xl font-black text-white placeholder:text-gray-600 border-none outline-none"
          placeholder="Example: Validate invite-link signup flow..."
        />

        <label class="mt-6 block">
          <textarea
            bind:value={description}
            use:autoGrow
            class="min-h-20 w-full overflow-hidden resize-none !bg-transparent px-0 text-[15px] leading-relaxed text-gray-300 placeholder:text-gray-600 border-none outline-none"
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
              <SearchableSelect id="editor-assign-dev" bind:value={assignDev} options={assigneeOptions} minimal={true} />
            </div>
          </label>

          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <UserCheck size={14} />
              Assign Tester
            </span>
            <div class="property-select">
              <SearchableSelect id="editor-assign-tester" bind:value={assignTester} options={assigneeOptions} minimal={true} />
            </div>
          </label>
        </div>

        <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
          <label class="space-y-2">
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <ClipboardCheck size={14} />
              Status
            </span>
            <div class="property-select">
              <SearchableSelect id="editor-status" bind:value={status} options={statusOptions} showSearch={false} minimal={true} />
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
            <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
              <ListChecks size={14} />
              Suite
            </span>
            <div class="property-select">
              <SearchableSelect id="editor-suite" bind:value={suiteId} options={suiteOptions} minimal={true} />
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
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
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
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
            placeholder="Expected state after this test finishes..."
          ></textarea>
        </label>
      </section>

      <section class="grid grid-cols-1 gap-5 border-b border-white/5 px-6 py-6 lg:grid-cols-3">
        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <Keyboard size={14} />
            Input
          </span>
          <textarea
            bind:value={input}
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
            placeholder="Data that tester must enter..."
          ></textarea>
        </label>

        <label>
          <span class="flex items-center gap-2 text-[12px] font-black uppercase tracking-widest text-gray-500">
            <CheckCircle2 size={14} />
            Expected Result
          </span>
          <textarea
            bind:value={expectedResult}
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
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
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
            placeholder="Actual behavior found during testing..."
          ></textarea>
        </label>
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
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
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
            use:autoGrow
            class="mt-3 min-h-20 w-full overflow-hidden resize-none rounded-xl border border-white/10 !bg-transparent px-3 py-2 text-sm font-medium leading-6 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20"
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
                    <button
                      type="button"
                      class="grid h-5 w-5 place-items-center rounded text-gray-500 hover:text-rose-400 transition-colors"
                      title="Remove"
                      on:click={() => removeAttachment(index)}
                    >
                      <X size={13} />
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if attachments.length < MAX_ATTACHMENTS}
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
        {:else}
          <p class="mt-4 text-xs text-gray-500">Maximum {MAX_ATTACHMENTS} attachments reached.</p>
        {/if}
      </section>

      <section class="px-6 py-6">
        <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <h2 class="text-base font-black text-gray-100">Test Case Steps</h2>
            <div class="property-select min-w-[104px]">
              <SearchableSelect id="editor-step-format" bind:value={stepFormat} options={stepFormatOptions} showSearch={false} minimal={true} />
            </div>
          </div>
          {#if stepFormat === "gherkin"}
            <div class="flex items-center gap-4 text-xs font-black">
              <button 
                type="button"
                class="transition-colors hover:text-gray-300 {gherkinView === 'raw' ? 'text-indigo-400' : 'text-gray-500'}"
                on:click={() => { gherkinView = 'raw'; convertToRaw(); }}
              >
                Raw
              </button>
              <button 
                type="button"
                class="transition-colors hover:text-gray-300 {gherkinView === 'steps' ? 'text-indigo-400' : 'text-gray-500'}"
                on:click={() => { gherkinView = 'steps'; convertToSteps(); }}
              >
                Steps
              </button>
            </div>
          {/if}
        </div>

        {#if stepFormat === "classic"}
          <div class="space-y-4">
            {#each classicSteps as step, index}
              <div class="grid items-start gap-3" style="grid-template-columns: 28px minmax(0,1fr) minmax(0,1fr) minmax(0,1fr) 72px;">
                <div class="pt-2 text-center text-sm font-black text-gray-400">{index + 1}</div>
                <textarea rows="1" bind:value={step.action} use:autoGrow class="min-h-[44px] w-full overflow-hidden resize-none rounded-lg border border-white/10 !bg-transparent px-3 py-2.5 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20" placeholder="Action"></textarea>
                <textarea rows="1" bind:value={step.data} use:autoGrow class="min-h-[44px] w-full overflow-hidden resize-none rounded-lg border border-white/10 !bg-transparent px-3 py-2.5 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20" placeholder="Data"></textarea>
                <textarea rows="1" bind:value={step.expected} use:autoGrow class="min-h-[44px] w-full overflow-hidden resize-none rounded-lg border border-white/10 !bg-transparent px-3 py-2.5 text-sm font-medium leading-5 text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20" placeholder="Expected result"></textarea>
                <div class="flex items-center gap-2 pt-1">
                  <button type="button" class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:text-rose-400" title="Delete step" on:click={() => removeClassicStep(index)}>
                    <Trash2 size={15} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
          <div class="mt-5 flex flex-wrap items-center gap-3">
            <button type="button" class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-gray-200 hover:border-white/20" on:click={addClassicStep}>
              <Plus size={15} />
              New step
            </button>
          </div>
        {:else}
          {#if gherkinView === 'raw'}
            <div class="relative min-h-40 w-full rounded-xl border border-white/10 bg-[#151e2e] text-sm font-medium leading-6 font-mono shadow-inner">
              <!-- Syntax highlighting overlay -->
              <div class="absolute inset-0 px-4 py-3 whitespace-pre-wrap break-words pointer-events-none" aria-hidden="true">
                <!-- Add a dummy zero-width space if empty to maintain height, or rely on min-h-40 -->
                {@html highlightGherkin(gherkinRaw) || ' '}
              </div>
              <!-- Invisible textarea for editing -->
              <textarea
                bind:value={gherkinRaw}
                use:autoGrow
                class="relative w-full h-full min-h-40 resize-none bg-transparent text-transparent caret-white px-4 py-3 outline-none border-none m-0 focus:ring-0"
                spellcheck="false"
                placeholder="Given preconditions...&#10;When actions...&#10;Then expected results..."
              ></textarea>
            </div>
          {:else}
            <div class="space-y-3">
              {#each gherkinSteps as step, index}
                <div class="grid items-center gap-3" style="grid-template-columns: 28px 110px minmax(0, 1fr) 32px;">
                  <div class="text-center text-sm font-black text-gray-400">{index + 1}</div>
                  <div class="property-select">
                    <SearchableSelect id={`editor-step-keyword-${index}`} bind:value={step.keyword} options={gherkinKeywordOptions} showSearch={false} minimal={true} />
                  </div>
                  <input bind:value={step.text} class="h-10 min-w-0 rounded-lg border border-white/10 !bg-transparent px-3 text-sm font-medium text-gray-200 outline-none placeholder:text-gray-600 focus:border-white/20" placeholder="Write step..." />
                  <button type="button" class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:text-rose-400" title="Delete step" on:click={() => removeGherkinStep(index)}>
                    <Trash2 size={15} />
                  </button>
                </div>
              {/each}
            </div>
            <button type="button" class="mt-5 inline-flex items-center gap-1.5 rounded-lg px-1 py-1 text-sm font-black text-indigo-400 hover:text-indigo-300" on:click={addGherkinStep}>
              <Plus size={15} />
              Add step
            </button>
          {/if}
        {/if}
      </section>
    </main>
  </form>
</div>

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
