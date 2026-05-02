<script lang="ts">
  import { Plus, Trash2, Image, MoreHorizontal } from "lucide-svelte";
  import SearchableSelect from "./SearchableSelect.svelte";
  import { _ } from "svelte-i18n";

  export let steps: any[] = [];
  export let format: "classic" | "gherkin" = "classic";
  export let readOnly: boolean = false;
  export let gherkinView: "raw" | "steps" = "steps";
  export let gherkinRaw: string = "";

  const gherkinKeywordOptions = [
    { value: "given", label: "Given" },
    { value: "when", label: "When" },
    { value: "then", label: "Then" },
    { value: "and", label: "And" },
  ];

  function addStep() {
    if (format === "classic") {
      steps = [...steps, { action: "", data: "", expected: "" }];
    } else {
      const next =
        steps.length % 3 === 0
          ? "given"
          : steps.length % 3 === 1
            ? "when"
            : "then";
      steps = [...steps, { keyword: next, text: "" }];
    }
  }

  function removeStep(index: number) {
    steps = steps.filter((_, i) => i !== index);
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

  function highlightGherkin(text: string) {
    if (!text) return "";
    return text
      .split("\n")
      .map((line) => {
        const match = line.match(/^(\s*)(Given|When|Then|And)\b(.*)/i);
        if (match) {
          const rest = match[3]
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
          return `${match[1]}<span class="text-sky-400 font-bold">${match[2]}</span><span class="text-gray-200">${rest}</span>`;
        }
        return `<span class="text-gray-200">${line.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")}</span>`;
      })
      .join("<br>");
  }

  function convertToRaw() {
    gherkinRaw = steps
      .filter((s) => s.text?.trim() || s.keyword)
      .map(
        (s) =>
          `${s.keyword.charAt(0).toUpperCase() + s.keyword.slice(1)} ${s.text}`,
      )
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
          return {
            keyword: firstWord as any,
            text: trimmed.substring(firstSpace + 1),
          };
        }
      }
      return { keyword: "given" as any, text: trimmed };
    });

    if (parsed.length > 0) {
      steps = parsed;
    } else {
      steps = [{ keyword: "given", text: "" }];
    }
  }
</script>

<div class="space-y-5">
  {#if format === "gherkin" && !readOnly}
    <div class="flex items-center justify-end gap-4 text-xs font-black">
      <button
        type="button"
        class="transition-colors hover:text-gray-300 {gherkinView === 'raw'
          ? 'text-indigo-400'
          : 'text-gray-500'}"
        on:click={() => {
          gherkinView = "raw";
          convertToRaw();
        }}
      >
        Raw
      </button>
      <button
        type="button"
        class="transition-colors hover:text-gray-300 {gherkinView === 'steps'
          ? 'text-indigo-400'
          : 'text-gray-500'}"
        on:click={() => {
          gherkinView = "steps";
          convertToSteps();
        }}
      >
        Steps
      </button>
    </div>
  {/if}

  {#if format === "classic"}
    {#each steps as step, index}
      <div
        class="grid gap-3"
        style={readOnly
          ? "grid-template-columns: 28px minmax(0,1fr) 76px;"
          : "grid-template-columns: 28px minmax(0,1fr) minmax(0,1fr) minmax(0,1fr) 72px;"}
      >
        <div
          class="pt-2 text-center text-sm font-black text-slate-400 dark:text-gray-500"
        >
          {index + 1}
        </div>

        {#if readOnly}
          <div class="space-y-2 py-1">
            <div class="text-sm font-medium text-slate-800 dark:text-gray-200">
              {step.action || "No action"}
            </div>
            {#if step.data}
              <div class="text-xs font-medium text-slate-500 dark:text-gray-400">
                <span class="mr-1 font-black uppercase text-slate-400"
                  >Data:</span
                >
                {step.data}
              </div>
            {/if}
            {#if step.expected}
              <div class="text-xs font-medium text-slate-500 dark:text-gray-400">
                <span class="mr-1 font-black uppercase text-slate-400"
                  >Expected:</span
                >
                {step.expected}
              </div>
            {/if}
          </div>
        {:else}
          <textarea
            rows="1"
            bind:value={step.action}
            use:autoGrow
            class="min-h-[38px] w-full overflow-hidden resize-none rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm font-medium leading-5 text-slate-800 outline-none placeholder:text-slate-400 focus:border-indigo-500 dark:border-white/10 dark:!bg-transparent dark:text-gray-200 dark:placeholder:text-gray-600 dark:focus:border-white/20"
            placeholder="Action"
          ></textarea>
          <textarea
            rows="1"
            bind:value={step.data}
            use:autoGrow
            class="min-h-[38px] w-full overflow-hidden resize-none rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm font-medium leading-5 text-slate-800 outline-none placeholder:text-slate-400 focus:border-indigo-500 dark:border-white/10 dark:!bg-transparent dark:text-gray-200 dark:placeholder:text-gray-600 dark:focus:border-white/20"
            placeholder="Data"
          ></textarea>
          <textarea
            rows="1"
            bind:value={step.expected}
            use:autoGrow
            class="min-h-[38px] w-full overflow-hidden resize-none rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm font-medium leading-5 text-slate-800 outline-none placeholder:text-slate-400 focus:border-indigo-500 dark:border-white/10 dark:!bg-transparent dark:text-gray-200 dark:placeholder:text-gray-600 dark:focus:border-white/20"
            placeholder="Expected result"
          ></textarea>
        {/if}

        <div class="flex items-start gap-2 pt-1">
          {#if readOnly}
          {:else}
            <button
              type="button"
              class="grid h-8 w-8 place-items-center rounded-md text-slate-500 hover:text-rose-600 dark:hover:text-rose-400"
              title="Delete step"
              on:click={() => removeStep(index)}
            >
              <Trash2 size={15} />
            </button>
          {/if}
        </div>
      </div>
    {/each}
  {:else if gherkinView === "raw" && !readOnly}
    <!-- Gherkin Raw View (only for editing) -->
    <div
      class="relative min-h-40 w-full rounded-xl border border-slate-200 bg-white font-mono text-sm font-medium leading-6 shadow-sm dark:border-white/10 dark:bg-[#151e2e] dark:shadow-inner"
    >
      <div
        class="pointer-events-none absolute inset-0 whitespace-pre-wrap break-words px-4 py-3"
        aria-hidden="true"
      >
        {@html highlightGherkin(gherkinRaw) || " "}
      </div>
      <textarea
        bind:value={gherkinRaw}
        use:autoGrow
        rows="1"
        class="relative m-0 h-full min-h-[38px] w-full resize-none border-none bg-transparent px-4 py-3 text-transparent caret-slate-800 outline-none focus:ring-0 dark:caret-white"
        spellcheck="false"
        placeholder="Given preconditions...&#10;When actions...&#10;Then expected results..."
      ></textarea>
    </div>
  {:else}
    <!-- Gherkin Steps View -->
    {#each steps as step, index}
      <div
        class="grid items-center gap-3"
        style={readOnly
          ? "grid-template-columns: 28px minmax(0,1fr) 76px;"
          : "grid-template-columns: 28px 104px minmax(0,1fr) 32px;"}
      >
        <div
          class="text-center text-sm font-black text-slate-400 dark:text-gray-500"
        >
          {index + 1}
        </div>

        {#if readOnly}
          <div class="py-2 text-sm font-medium text-slate-800 dark:text-gray-200">
            <span
              class="mr-2 font-black uppercase text-indigo-600 dark:text-indigo-400"
              >{step.keyword}</span
            >
            {step.text || "No text"}
          </div>
        {:else}
          <div class="property-select">
            <SearchableSelect
              id={`step-keyword-${index}`}
              bind:value={step.keyword}
              options={gherkinKeywordOptions}
              showSearch={false}
              minimal={true}
            />
          </div>
          <input
            bind:value={step.text}
            class="h-9 min-w-0 rounded-lg border border-slate-200 bg-white px-3 text-sm font-medium text-slate-800 outline-none placeholder:text-slate-400 focus:border-indigo-500 dark:border-white/10 dark:!bg-transparent dark:text-gray-200 dark:placeholder:text-gray-600 dark:focus:border-white/20"
            placeholder="Write step..."
          />
        {/if}

        <div class="flex items-center gap-2">
          {#if readOnly}
            <!-- Actions for readOnly gherkin -->
          {:else}
            <button
              type="button"
              class="grid h-8 w-8 place-items-center rounded-md text-slate-500 hover:text-rose-600 dark:hover:text-rose-400"
              title="Delete step"
              on:click={() => removeStep(index)}
            >
              <Trash2 size={15} />
            </button>
          {/if}
        </div>
      </div>
    {/each}
  {/if}

  {#if !readOnly}
    <div class="mt-4 flex flex-wrap items-center gap-3">
      <button
        type="button"
        class="inline-flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-2 text-sm font-black text-gray-200 hover:border-white/20"
        on:click={addStep}
      >
        <Plus size={15} />
        {$_("testCases__add_step")}
      </button>
    </div>
  {/if}
</div>

<style>
  :global(.property-select .property-trigger-btn) {
    background-color: transparent !important;
    border-width: 1px !important;
    border-style: solid !important;
    border-color: rgba(255, 255, 255, 0.1) !important;
    color: #9ca3af !important;
    border-radius: 9999px !important;
    height: 2.25rem !important;
    padding-left: 0.75rem !important;
    padding-right: 0.75rem !important;
    font-size: 13px !important;
    box-shadow: none !important;
  }
</style>
