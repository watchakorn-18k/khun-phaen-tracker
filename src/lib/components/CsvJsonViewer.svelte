<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { Table, X, Upload, ArrowUpDown, ArrowUp, ArrowDown, Search, Copy, Check, BarChart3, Info } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();
  const STORAGE_KEY = "csv-json-viewer";

  type SortDir = "asc" | "desc" | null;

  let rawInput = $state("");
  let inputMode = $state<"csv" | "json">("csv");
  let delimiter = $state(",");
  let columns = $state<string[]>([]);
  let rows = $state<Record<string, string>[]>([]);
  let sortCol = $state<string | null>(null);
  let sortDir = $state<SortDir>(null);
  let filterCol = $state<string | null>(null);
  let filterVal = $state("");
  let globalSearch = $state("");
  let copied = $state(false);
  let statsOpen = $state(false);
  let fileName = $state("");
  let parseError = $state("");
  let showHelp = $state(false);

  function parseCSV(text: string, delim: string): { columns: string[]; rows: Record<string, string>[] } {
    const lines = text.trim().split(/\r?\n/);
    if (lines.length === 0) return { columns: [], rows: [] };

    function splitLine(line: string): string[] {
      const result: string[] = [];
      let current = "";
      let inQuotes = false;
      for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (ch === '"') {
          if (inQuotes && i + 1 < line.length && line[i + 1] === '"') {
            current += '"';
            i++;
          } else {
            inQuotes = !inQuotes;
          }
        } else if (ch === delim.charAt(0) && !inQuotes) {
          result.push(current.trim());
          current = "";
        } else {
          current += ch;
        }
      }
      result.push(current.trim());
      return result;
    }

    const cols = splitLine(lines[0]);
    const parsedRows: Record<string, string>[] = [];
    for (let i = 1; i < lines.length; i++) {
      if (!lines[i].trim()) continue;
      const vals = splitLine(lines[i]);
      const row: Record<string, string> = {};
      for (let j = 0; j < cols.length; j++) {
        row[cols[j]] = vals[j] ?? "";
      }
      parsedRows.push(row);
    }
    return { columns: cols, rows: parsedRows };
  }

  function parseJSON(text: string): { columns: string[]; rows: Record<string, string>[] } {
    const data = JSON.parse(text);
    let arr: any[] = [];
    if (Array.isArray(data)) {
      arr = data;
    } else if (typeof data === "object" && data !== null) {
      arr = [data];
    } else {
      throw new Error("JSON must be an array or object");
    }
    if (arr.length === 0) return { columns: [], rows: [] };

    const colSet = new Set<string>();
    for (const item of arr) {
      if (typeof item === "object" && item !== null) {
        for (const key of Object.keys(item)) colSet.add(key);
      }
    }
    const cols = Array.from(colSet);
    const parsedRows = arr.map((item) => {
      if (typeof item !== "object" || item === null) {
        const row: Record<string, string> = {};
        cols.forEach((c) => (row[c] = String(item)));
        return row;
      }
      const row: Record<string, string> = {};
      for (const c of cols) {
        const v = item[c];
        row[c] = v === undefined || v === null ? "" : typeof v === "object" ? JSON.stringify(v) : String(v);
      }
      return row;
    });
    return { columns: cols, rows: parsedRows };
  }

  function parse() {
    parseError = "";
    if (!rawInput.trim()) {
      columns = [];
      rows = [];
      return;
    }
    try {
      if (inputMode === "csv") {
        const result = parseCSV(rawInput, delimiter);
        columns = result.columns;
        rows = result.rows;
      } else {
        const result = parseJSON(rawInput);
        columns = result.columns;
        rows = result.rows;
      }
    } catch (e: any) {
      parseError = e.message || "Parse error";
      columns = [];
      rows = [];
    }
  }

  function handleFileDrop(e: DragEvent) {
    e.preventDefault();
    const file = e.dataTransfer?.files[0];
    if (!file) return;
    readFile(file);
  }

  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    readFile(file);
    input.value = "";
  }

  function readFile(file: File) {
    fileName = file.name;
    const reader = new FileReader();
    reader.onload = () => {
      const text = reader.result as string;
      if (file.name.endsWith(".json") || file.name.endsWith(".jsonl")) {
        inputMode = "json";
      } else {
        inputMode = "csv";
      }
      rawInput = text;
      parse();
    };
    reader.readAsText(file);
  }

  function handlePaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData("text");
    if (!text) return;
    rawInput = text;
    parse();
  }

  let filteredRows = $derived.by(() => {
    let result = rows;
    if (globalSearch) {
      const q = globalSearch.toLowerCase();
      result = result.filter((row) =>
        columns.some((col) => String(row[col] ?? "").toLowerCase().includes(q))
      );
    }
    if (filterCol && filterVal) {
      const q = filterVal.toLowerCase();
      result = result.filter((row) =>
        String(row[filterCol!] ?? "").toLowerCase().includes(q)
      );
    }
    if (sortCol && sortDir) {
      result = [...result].sort((a, b) => {
        const va = String(a[sortCol!] ?? "");
        const vb = String(b[sortCol!] ?? "");
        const na = Number(va);
        const nb = Number(vb);
        if (!isNaN(na) && !isNaN(nb)) {
          return sortDir === "asc" ? na - nb : nb - na;
        }
        return sortDir === "asc" ? va.localeCompare(vb) : vb.localeCompare(va);
      });
    }
    return result;
  });

  type ColStats = {
    count: number;
    unique: number;
    numeric: boolean;
    mean: string;
    median: string;
    min: string;
    max: string;
    topValues: { value: string; count: number }[];
  };

  let colStats = $derived.by(() => {
    const stats: Record<string, ColStats> = {};
    for (const col of columns) {
      const values = rows.map((r) => String(r[col] ?? "")).filter((v) => v !== "");
      const unique = new Set(values);
      const nums = values.map(Number).filter((n) => !isNaN(n));
      const isNumeric = nums.length > values.length * 0.5;

      let mean = "-";
      let median = "-";
      let min = "-";
      let max = "-";

      if (isNumeric && nums.length > 0) {
        nums.sort((a, b) => a - b);
        const sum = nums.reduce((a, b) => a + b, 0);
        mean = (sum / nums.length).toFixed(2);
        const mid = Math.floor(nums.length / 2);
        median = nums.length % 2 !== 0
          ? nums[mid].toFixed(2)
          : ((nums[mid - 1] + nums[mid]) / 2).toFixed(2);
        min = String(nums[0]);
        max = String(nums[nums.length - 1]);
      }

      const freq = new Map<string, number>();
      for (const v of values) freq.set(v, (freq.get(v) || 0) + 1);
      const topValues = Array.from(freq.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 5)
        .map(([value, count]) => ({ value, count }));

      stats[col] = {
        count: values.length,
        unique: unique.size,
        numeric: isNumeric,
        mean,
        median,
        min,
        max,
        topValues,
      };
    }
    return stats;
  });

  function toggleSort(col: string) {
    if (sortCol === col) {
      if (sortDir === "asc") sortDir = "desc";
      else if (sortDir === "desc") { sortCol = null; sortDir = null; }
    } else {
      sortCol = col;
      sortDir = "asc";
    }
  }

  function sortIcon(col: string) {
    if (sortCol !== col) return ArrowUpDown;
    return sortDir === "asc" ? ArrowUp : ArrowDown;
  }

  function handleCopy() {
    if (filteredRows.length === 0) return;
    const header = columns.join("\t");
    const body = filteredRows.map((r) => columns.map((c) => String(r[c] ?? "")).join("\t")).join("\n");
    navigator.clipboard.writeText(header + "\n" + body);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleClose();
  }

  function handleClose() {
    dispatch("close");
  }

  function exportAs(format: "csv" | "json") {
    if (filteredRows.length === 0) return;
    let content: string;
    let mime: string;
    let ext: string;

    if (format === "csv") {
      const header = columns.join(",");
      const body = filteredRows.map((r) =>
        columns.map((c) => {
          const v = String(r[c] ?? "");
          return v.includes(",") || v.includes('"') || v.includes("\n") ? `"${v.replace(/"/g, '""')}"` : v;
        }).join(",")
      ).join("\n");
      content = header + "\n" + body;
      mime = "text/csv";
      ext = "csv";
    } else {
      const arr = filteredRows.map((r) => {
        const obj: Record<string, any> = {};
        for (const c of columns) {
          const v = String(r[c] ?? "");
          const n = Number(v);
          obj[c] = v !== "" && !isNaN(n) ? n : v;
        }
        return obj;
      });
      content = JSON.stringify(arr, null, 2);
      mime = "application/json";
      ext = "json";
    }

    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `data-export.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  onMount(() => {
    if (browser) {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        try {
          const d = JSON.parse(saved);
          rawInput = d.rawInput || "";
          inputMode = d.inputMode || "csv";
          delimiter = d.delimiter || ",";
          parse();
        } catch {}
      }
    }
  });

  $effect(() => {
    if (browser) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ rawInput, inputMode, delimiter }));
    }
  });
</script>

{#if browser}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-6xl w-full h-[90vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-cyan-100 dark:bg-cyan-900/30 rounded-lg">
            <Table class="text-cyan-600 dark:text-cyan-400" size={20} />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {$_('csvJsonViewer__title') || 'CSV / JSON Viewer'}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {$_('csvJsonViewer__subtitle') || 'Paste data or drop a file to view as a table'}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={() => (showHelp = !showHelp)}
            class="p-2 rounded-lg transition-all {showHelp ? 'text-cyan-600 dark:text-cyan-400 bg-cyan-100 dark:bg-cyan-900/30' : 'text-gray-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
            title="Help"
          >
            <Info size={18} />
          </button>
          {#if rows.length > 0}
            <button
              onclick={() => (statsOpen = !statsOpen)}
              class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {statsOpen
                ? 'bg-cyan-100 dark:bg-cyan-900/30 text-cyan-600 dark:text-cyan-400 border border-cyan-300 dark:border-cyan-700'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 border border-transparent hover:border-gray-300 dark:hover:border-gray-600'}"
            >
              <BarChart3 size={14} class="inline -mt-0.5 mr-1" />
              {$_('csvJsonViewer__stats') || 'Stats'}
            </button>
          {/if}
          <button
            onclick={handleClose}
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X size={20} />
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        <!-- Input Area (shown when no data or explicitly) -->
        {#if rows.length === 0 && !parseError || rawInput === ""}
          <!-- Drop Zone -->
          <div
            class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-xl p-8 text-center transition-colors hover:border-cyan-400 dark:hover:border-cyan-500"
            ondragover={(e) => { e.preventDefault(); (e.currentTarget as HTMLElement).classList.add('border-cyan-400', 'dark:border-cyan-500', 'bg-cyan-50', 'dark:bg-cyan-900/10'); }}
            ondragleave={(e) => { (e.currentTarget as HTMLElement).classList.remove('border-cyan-400', 'dark:border-cyan-500', 'bg-cyan-50', 'dark:bg-cyan-900/10'); }}
            ondrop={handleFileDrop}
          >
            <Upload class="mx-auto text-gray-400 dark:text-gray-500 mb-3" size={32} />
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
              {$_('csvJsonViewer__dropzone') || 'Drag & drop a CSV or JSON file here'}
            </p>
            <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
              {$_('csvJsonViewer__or') || 'or'}
            </p>
            <label class="inline-block mt-2 px-4 py-2 rounded-lg text-sm font-bold bg-cyan-100 dark:bg-cyan-900/30 text-cyan-600 dark:text-cyan-400 cursor-pointer hover:opacity-80 transition-all">
              {$_('csvJsonViewer__browse') || 'Browse file'}
              <input type="file" accept=".csv,.json,.jsonl,.tsv,.txt" class="hidden" onchange={handleFileSelect} />
            </label>
          </div>

          <!-- Input Mode Toggle -->
          <div class="flex items-center gap-3">
            <div class="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
              <button
                onclick={() => (inputMode = "csv")}
                class="px-3 py-1.5 rounded-md text-xs font-bold transition-all {inputMode === 'csv'
                  ? 'bg-white dark:bg-gray-600 text-cyan-600 dark:text-cyan-400 shadow-sm'
                  : 'text-gray-500 dark:text-gray-400'}"
              >CSV</button>
              <button
                onclick={() => (inputMode = "json")}
                class="px-3 py-1.5 rounded-md text-xs font-bold transition-all {inputMode === 'json'
                  ? 'bg-white dark:bg-gray-600 text-cyan-600 dark:text-cyan-400 shadow-sm'
                  : 'text-gray-500 dark:text-gray-400'}"
              >JSON</button>
            </div>
            {#if inputMode === "csv"}
              <div class="flex items-center gap-1.5 text-xs text-gray-500">
                <span>{$_('csvJsonViewer__delimiter') || 'Delimiter'}:</span>
                {#each [",", ";", "\t", "|"] as d}
                  <button
                    onclick={() => (delimiter = d)}
                    class="px-2 py-1 rounded-md text-xs font-mono font-bold transition-all {delimiter === d
                      ? 'bg-cyan-100 dark:bg-cyan-900/30 text-cyan-600 dark:text-cyan-400'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                  >
                    {d === "\t" ? "TAB" : d}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Paste Area -->
          <textarea
            bind:value={rawInput}
            class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none resize-y min-h-[160px] max-h-[300px] focus:border-cyan-400 dark:focus:border-cyan-600 transition-colors placeholder-gray-400"
            placeholder={inputMode === "csv"
              ? "name,age,city\nAlice,30,Bangkok\nBob,25,Tokyo"
              : '[{"name": "Alice", "age": 30, "city": "Bangkok"}]'}
            onpaste={handlePaste}
            onchange={parse}
          ></textarea>

          {#if rawInput.trim()}
            <button
              onclick={parse}
              class="px-4 py-2 rounded-xl text-sm font-bold text-white bg-cyan-600 hover:bg-cyan-700 transition-all"
            >
              {$_('csvJsonViewer__parse') || 'Parse Data'}
            </button>
          {/if}
        {/if}

        <!-- Parse Error -->
        {#if parseError}
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono">
            {parseError}
          </div>
        {/if}

        <!-- Stats Panel -->
        {#if statsOpen && rows.length > 0}
          <div class="bg-cyan-50 dark:bg-cyan-900/15 border border-cyan-200 dark:border-cyan-800/50 rounded-xl p-4 space-y-3">
            <h4 class="text-sm font-bold text-cyan-700 dark:text-cyan-300 uppercase tracking-wider">
              {$_('csvJsonViewer__stats') || 'Statistics'} — {rows.length} {$_('csvJsonViewer__rows') || 'rows'} &middot; {columns.length} {$_('csvJsonViewer__columns') || 'columns'}
            </h4>
            <div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
              {#each columns as col}
                {@const s = colStats[col]}
                <div class="bg-white dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700 space-y-1.5">
                  <p class="text-xs font-black text-cyan-600 dark:text-cyan-400 uppercase truncate">{col}</p>
                  <div class="grid grid-cols-2 gap-x-3 gap-y-0.5 text-[11px]">
                    <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_count') || 'Count'}</span>
                    <span class="font-mono font-bold text-gray-700 dark:text-gray-300">{s?.count ?? 0}</span>
                    <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_unique') || 'Unique'}</span>
                    <span class="font-mono font-bold text-gray-700 dark:text-gray-300">{s?.unique ?? 0}</span>
                    {#if s?.numeric}
                      <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_mean') || 'Mean'}</span>
                      <span class="font-mono font-bold text-cyan-600 dark:text-cyan-400">{s.mean}</span>
                      <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_median') || 'Median'}</span>
                      <span class="font-mono font-bold text-cyan-600 dark:text-cyan-400">{s.median}</span>
                      <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_min') || 'Min'}</span>
                      <span class="font-mono font-bold text-gray-700 dark:text-gray-300">{s.min}</span>
                      <span class="text-gray-500 dark:text-gray-400">{$_('csvJsonViewer__stat_max') || 'Max'}</span>
                      <span class="font-mono font-bold text-gray-700 dark:text-gray-300">{s.max}</span>
                    {/if}
                  </div>
                  {#if s?.topValues && s.topValues.length > 0}
                    <div class="mt-1 pt-1 border-t border-gray-100 dark:border-gray-700 space-y-0.5">
                      <p class="text-[10px] font-bold text-gray-400 uppercase">{$_('csvJsonViewer__stat_top') || 'Top values'}</p>
                      {#each s.topValues.slice(0, 3) as tv}
                        <div class="flex items-center justify-between text-[10px]">
                          <span class="truncate max-w-[120px] font-mono text-gray-600 dark:text-gray-300" title={tv.value}>{tv.value || "(empty)"}</span>
                          <span class="font-bold text-gray-400">{tv.count}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Table View -->
        {#if rows.length > 0}
          <!-- Toolbar -->
          <div class="flex items-center gap-2 flex-wrap">
            <!-- Global Search -->
            <div class="relative flex-1 min-w-[200px]">
              <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
              <input
                type="text"
                bind:value={globalSearch}
                class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg pl-9 pr-3 py-2 text-sm text-gray-800 dark:text-gray-200 outline-none focus:border-cyan-400 dark:focus:border-cyan-600 transition-colors placeholder-gray-400"
                placeholder={$_('csvJsonViewer__search') || 'Search all columns...'}
              />
            </div>

            <!-- Column Filter -->
            <select
              class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-800 dark:text-gray-200 outline-none"
              onchange={(e) => { filterCol = (e.target as HTMLSelectElement).value || null; }}
            >
              <option value="">{$_('csvJsonViewer__filter_col') || 'Filter column...'}</option>
              {#each columns as col}
                <option value={col} selected={filterCol === col}>{col}</option>
              {/each}
            </select>

            {#if filterCol}
              <input
                type="text"
                bind:value={filterVal}
                class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-800 dark:text-gray-200 outline-none w-40 focus:border-cyan-400 dark:focus:border-cyan-600 transition-colors placeholder-gray-400"
                placeholder={$_('csvJsonViewer__filter_val') || 'Filter value...'}
              />
            {/if}

            <div class="flex items-center gap-1 ml-auto">
              <button
                onclick={handleCopy}
                class="p-2 text-gray-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all"
                title={$_('csvJsonViewer__copy') || 'Copy table'}
              >
                {#if copied}
                  <Check size={16} class="text-green-500" />
                {:else}
                  <Copy size={16} />
                {/if}
              </button>
              <button
                onclick={() => exportAs("csv")}
                class="px-3 py-1.5 rounded-lg text-xs font-bold bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/30 hover:text-cyan-600 dark:hover:text-cyan-400 transition-all"
              >
                CSV
              </button>
              <button
                onclick={() => exportAs("json")}
                class="px-3 py-1.5 rounded-lg text-xs font-bold bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/30 hover:text-cyan-600 dark:hover:text-cyan-400 transition-all"
              >
                JSON
              </button>
              <button
                onclick={() => { rawInput = ""; columns = []; rows = []; parseError = ""; fileName = ""; }}
                class="px-3 py-1.5 rounded-lg text-xs font-bold bg-gray-100 dark:bg-gray-700 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all"
              >
                {$_('csvJsonViewer__clear') || 'Clear'}
              </button>
            </div>
          </div>

          <!-- File info -->
          {#if fileName}
            <div class="text-xs text-gray-400">
              {fileName} &middot; {rows.length} {$_('csvJsonViewer__rows') || 'rows'} &middot; {columns.length} {$_('csvJsonViewer__columns') || 'columns'}
              {#if filteredRows.length !== rows.length}
                &middot; {$_('csvJsonViewer__showing') || 'showing'} {filteredRows.length}/{rows.length}
              {/if}
            </div>
          {/if}

          <!-- Table -->
          <div class="border border-gray-200 dark:border-gray-700 rounded-xl overflow-hidden">
            <div class="overflow-auto max-h-[calc(90vh-340px)]">
              <table class="w-full text-sm">
                <thead class="bg-gray-50 dark:bg-gray-900/50 sticky top-0 z-10">
                  <tr>
                    <th class="px-3 py-2.5 text-[10px] font-black text-gray-400 uppercase tracking-wider border-b border-gray-200 dark:border-gray-700 w-10 text-center">#</th>
                    {#each columns as col}
                      <th class="px-3 py-2.5 text-left text-xs font-bold text-gray-600 dark:text-gray-300 uppercase tracking-wider border-b border-gray-200 dark:border-gray-700 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors select-none whitespace-nowrap" onclick={() => toggleSort(col)}>
                        <div class="flex items-center gap-1.5">
                          <span class="truncate max-w-[180px]">{col}</span>
                          {#if sortCol === col}
                            <svelte:component this={sortIcon(col)} size={12} class={sortDir === "asc" ? "text-cyan-600 dark:text-cyan-400" : "text-cyan-600 dark:text-cyan-400"} />
                          {:else}
                            <ArrowUpDown size={12} class="text-gray-300 dark:text-gray-600" />
                          {/if}
                        </div>
                      </th>
                    {/each}
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100 dark:divide-gray-700/50">
                  {#each filteredRows as row, i}
                    <tr class="hover:bg-cyan-50/50 dark:hover:bg-cyan-900/10 transition-colors">
                      <td class="px-3 py-2 text-[10px] font-mono text-gray-400 text-center">{i + 1}</td>
                      {#each columns as col}
                        <td class="px-3 py-2 text-gray-700 dark:text-gray-300 font-mono text-xs max-w-[300px] truncate" title={String(row[col] ?? "")}>
                          {String(row[col] ?? "")}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>

              {#if filteredRows.length === 0}
                <div class="text-center py-12 text-sm text-gray-400 dark:text-gray-500">
                  {$_('csvJsonViewer__no_results') || 'No matching rows found'}
                </div>
              {/if}
            </div>
          </div>

          {#if filteredRows.length > 0}
            <div class="text-xs text-gray-400 text-right">
              {$_('csvJsonViewer__showing') || 'Showing'} {filteredRows.length} {$_('csvJsonViewer__of') || 'of'} {rows.length} {$_('csvJsonViewer__rows') || 'rows'}
            </div>
          {/if}
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
        <div class="flex items-center gap-2">
          <span class="inline-block w-2 h-2 rounded-full {parseError ? 'bg-red-500' : rows.length > 0 ? 'bg-green-500' : 'bg-gray-400'}"></span>
          <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
            {parseError
              ? ($_('csvJsonViewer__parse_error') || 'Parse error')
              : rows.length > 0
                ? `${rows.length} ${$_('csvJsonViewer__rows') || 'rows'}`
                : ($_('csvJsonViewer__ready') || 'Ready')}
          </span>
        </div>
        {#if fileName}
          <div class="text-[10px] text-gray-400 font-mono truncate max-w-[200px]">
            {fileName}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes modal-in {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(-10px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .animate-modal-in {
    animation: modal-in 0.2s ease-out;
  }
</style>
