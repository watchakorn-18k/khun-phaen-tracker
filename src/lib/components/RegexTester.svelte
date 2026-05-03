<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { Regex, X, Copy, Check, Info } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();

  const STORAGE_KEY = "regex-tester";

  let pattern = $state("");
  let flags = $state("g");
  let testString = $state("");
  let error = $state("");
  let copied = $state(false);
  let showHelp = $state(false);

  let matches = $derived.by(() => {
    if (!pattern || !testString) return [];
    try {
      const regex = new RegExp(pattern, flags);
      const results: { match: string; index: number; groups: Record<string, string> | undefined }[] = [];
      let m;
      if (flags.includes("g")) {
        while ((m = regex.exec(testString)) !== null) {
          results.push({ match: m[0], index: m.index, groups: m.groups });
          if (m[0].length === 0) regex.lastIndex++;
        }
      } else {
        m = regex.exec(testString);
        if (m) results.push({ match: m[0], index: m.index, groups: m.groups });
      }
      error = "";
      return results;
    } catch (e: any) {
      error = e.message;
      return [];
    }
  });

  let highlightedHtml = $derived.by(() => {
    if (!pattern || !testString || error) return escapeHtml(testString);
    try {
      const regex = new RegExp(pattern, flags);
      let result = "";
      let lastIndex = 0;
      let m;
      const globalRegex = flags.includes("g") ? regex : new RegExp(pattern, flags + "g");

      while ((m = globalRegex.exec(testString)) !== null) {
        result += escapeHtml(testString.slice(lastIndex, m.index));
        result += `<mark class="bg-yellow-300 dark:bg-yellow-600/60 text-inherit rounded-sm px-0.5">${escapeHtml(m[0])}</mark>`;
        lastIndex = m.index + m[0].length;
        if (m[0].length === 0) globalRegex.lastIndex++;
      }
      result += escapeHtml(testString.slice(lastIndex));
      return result;
    } catch {
      return escapeHtml(testString);
    }
  });

  const commonPatterns = [
    { label: "Email", pattern: "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}" },
    { label: "URL", pattern: "https?://[\\w\\-]+(\\.[\\w\\-]+)+[\\w\\-.,@?^=%&:/~+#]*" },
    { label: "Phone (TH)", pattern: "0[689]\\d{8}" },
    { label: "IPv4", pattern: "\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b" },
    { label: "Date (YYYY-MM-DD)", pattern: "\\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\\d|3[01])" },
    { label: "HTML Tag", pattern: "<\\/?[a-zA-Z][\\w\\-]*(?:\\s[^>]*)?\\/?>" },
  ];

  function escapeHtml(str: string): string {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleClose();
  }

  function handleClose() {
    dispatch("close");
  }

  function handleCopy() {
    navigator.clipboard.writeText(`/${pattern}/${flags}`);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function toggleFlag(flag: string) {
    if (flags.includes(flag)) {
      flags = flags.replace(flag, "");
    } else {
      flags += flag;
    }
  }

  onMount(() => {
    if (browser) {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        try {
          const data = JSON.parse(saved);
          pattern = data.pattern || "";
          flags = data.flags || "g";
          testString = data.testString || "";
        } catch {}
      }
    }
  });

  $effect(() => {
    if (browser) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ pattern, flags, testString }));
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
      class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-5xl w-full h-[85vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-purple-100 dark:bg-purple-900/30 rounded-lg">
            <Regex class="text-purple-600 dark:text-purple-400" size={20} />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {$_('regexTester__title') || 'Regex Tester'}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {$_('regexTester__subtitle') || 'Test regex patterns with live matching'}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={() => (showHelp = !showHelp)}
            class="p-2 rounded-lg transition-all {showHelp ? 'text-purple-600 dark:text-purple-400 bg-purple-100 dark:bg-purple-900/30' : 'text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
            title="Help"
          >
            <Info size={18} />
          </button>
          <button
            onclick={handleCopy}
            class="p-2 text-gray-400 hover:text-purple-600 dark:hover:text-purple-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all"
            title={$_('regexTester__copy') || 'Copy regex'}
          >
            {#if copied}
              <Check size={18} class="text-green-500" />
            {:else}
              <Copy size={18} />
            {/if}
          </button>
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
        <!-- Help Guide -->
        {#if showHelp}
          <div class="bg-purple-50 dark:bg-purple-900/15 border border-purple-200 dark:border-purple-800/50 rounded-xl p-4 space-y-3">
            <h4 class="text-sm font-bold text-purple-700 dark:text-purple-300 uppercase tracking-wider">Regex Cheat Sheet</h4>
            <div class="grid grid-cols-2 gap-x-6 gap-y-1 text-[12px]">
              <div class="space-y-1">
                <p class="font-bold text-purple-600 dark:text-purple-400 mt-1 mb-0.5">อักขระพิเศษ</p>
                <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">.</code><span class="text-gray-600 dark:text-gray-400">ตัวอักษรอะไรก็ได้ 1 ตัว</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">\d</code><span class="text-gray-600 dark:text-gray-400">ตัวเลข 0-9</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">\w</code><span class="text-gray-600 dark:text-gray-400">ตัวอักษร ตัวเลข หรือ _</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">\s</code><span class="text-gray-600 dark:text-gray-400">ช่องว่าง, tab, newline</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">\b</code><span class="text-gray-600 dark:text-gray-400">ขอบเขตคำ</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">\</code><span class="text-gray-600 dark:text-gray-400">escape อักขระพิเศษ</span>
                </div>
                <p class="font-bold text-purple-600 dark:text-purple-400 mt-2 mb-0.5">Quantifiers</p>
                <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">*</code><span class="text-gray-600 dark:text-gray-400">0 ครั้งขึ้นไป</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">+</code><span class="text-gray-600 dark:text-gray-400">1 ครั้งขึ้นไป</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">?</code><span class="text-gray-600 dark:text-gray-400">0 หรือ 1 ครั้ง</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">{'{n}'}</code><span class="text-gray-600 dark:text-gray-400">ตรง n ครั้ง</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">{'{n,m}'}</code><span class="text-gray-600 dark:text-gray-400">ตรง n ถึง m ครั้ง</span>
                </div>
              </div>
              <div class="space-y-1">
                <p class="font-bold text-purple-600 dark:text-purple-400 mt-1 mb-0.5">Groups & Ranges</p>
                <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">[abc]</code><span class="text-gray-600 dark:text-gray-400">ตัวอักษร a, b หรือ c</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">[a-z]</code><span class="text-gray-600 dark:text-gray-400">ตัวอักษร a ถึง z</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">[^abc]</code><span class="text-gray-600 dark:text-gray-400">ยกเว้น a, b, c</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">(abc)</code><span class="text-gray-600 dark:text-gray-400">กลุ่ม capture</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">(a|b)</code><span class="text-gray-600 dark:text-gray-400">a หรือ b</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">(?&lt;name&gt;)</code><span class="text-gray-600 dark:text-gray-400">ตั้งชื่อกลุ่ม</span>
                </div>
                <p class="font-bold text-purple-600 dark:text-purple-400 mt-2 mb-0.5">Anchors</p>
                <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">^</code><span class="text-gray-600 dark:text-gray-400">จุดเริ่มต้น string/บรรทัด</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">$</code><span class="text-gray-600 dark:text-gray-400">จุดสิ้นสุด string/บรรทัด</span>
                </div>
                <p class="font-bold text-purple-600 dark:text-purple-400 mt-2 mb-0.5">Flags</p>
                <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">g</code><span class="text-gray-600 dark:text-gray-400">หาทั้งหมด (ไม่หยุดที่ตัวแรก)</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">i</code><span class="text-gray-600 dark:text-gray-400">ไม่สน case (A = a)</span>
                  <code class="text-purple-700 dark:text-purple-300 font-mono font-bold">m</code><span class="text-gray-600 dark:text-gray-400">^ $ ตรงทุกบรรทัด</span>
                </div>
              </div>
            </div>
          </div>
        {/if}
        <!-- Pattern Input -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('regexTester__pattern') || 'Pattern'}
          </label>
          <div class="flex items-center gap-2">
            <div class="flex-1 flex items-center bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl overflow-hidden focus-within:border-purple-400 dark:focus-within:border-purple-600 transition-colors">
              <span class="px-3 text-purple-500 font-mono font-bold text-lg">/</span>
              <input
                type="text"
                bind:value={pattern}
                class="flex-1 bg-transparent py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none placeholder-gray-400"
                placeholder={$_('regexTester__pattern_placeholder') || 'Enter regex pattern...'}
              />
              <span class="px-2 text-purple-500 font-mono font-bold text-lg">/</span>
              <input
                type="text"
                bind:value={flags}
                class="w-16 bg-transparent py-2.5 text-sm font-mono text-purple-600 dark:text-purple-400 outline-none"
                placeholder="g"
              />
            </div>
          </div>

          <!-- Flags Toggles -->
          <div class="flex items-center gap-2 mt-2 flex-wrap">
            {#each [
              { flag: "g", label: "Global" },
              { flag: "i", label: "Case insensitive" },
              { flag: "m", label: "Multiline" },
              { flag: "s", label: "Dotall" },
              { flag: "u", label: "Unicode" },
            ] as f}
              <button
                onclick={() => toggleFlag(f.flag)}
                class="px-2.5 py-1 rounded-lg text-[11px] font-bold transition-all {flags.includes(f.flag)
                  ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 border border-purple-300 dark:border-purple-700'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 border border-transparent hover:border-gray-300 dark:hover:border-gray-600'}"
              >
                {f.flag} — {f.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Error -->
        {#if error}
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono">
            {error}
          </div>
        {/if}

        <!-- Common Patterns -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('regexTester__common') || 'Common Patterns'}
          </label>
          <div class="flex flex-wrap gap-2">
            {#each commonPatterns as p}
              <button
                onclick={() => (pattern = p.pattern)}
                class="px-3 py-1.5 rounded-lg text-xs font-semibold bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-purple-100 dark:hover:bg-purple-900/30 hover:text-purple-600 dark:hover:text-purple-400 border border-transparent hover:border-purple-300 dark:hover:border-purple-700 transition-all"
              >
                {p.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Test String -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('regexTester__test_string') || 'Test String'}
          </label>
          <textarea
            bind:value={testString}
            class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none resize-y min-h-[100px] focus:border-purple-400 dark:focus:border-purple-600 transition-colors placeholder-gray-400"
            placeholder={$_('regexTester__test_placeholder') || 'Enter text to test against...'}
          ></textarea>
        </div>

        <!-- Highlighted Result -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('regexTester__result') || 'Result'}
          </label>
          <div class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 min-h-[80px] whitespace-pre-wrap break-all leading-relaxed">
            {@html highlightedHtml || '<span class="text-gray-400">-</span>'}
          </div>
        </div>

        <!-- Match List -->
        {#if matches.length > 0}
          <div>
            <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
              {$_('regexTester__matches') || 'Matches'} ({matches.length})
            </label>
            <div class="space-y-1.5 max-h-48 overflow-y-auto">
              {#each matches as m, i}
                <div class="flex items-center gap-3 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2">
                  <span class="text-[10px] font-bold text-purple-500 bg-purple-100 dark:bg-purple-900/30 px-2 py-0.5 rounded-md shrink-0">
                    #{i + 1}
                  </span>
                  <code class="text-sm text-gray-800 dark:text-gray-200 truncate flex-1">"{m.match}"</code>
                  <span class="text-[10px] text-gray-400 font-mono shrink-0">
                    index: {m.index}
                  </span>
                  {#if m.groups && Object.keys(m.groups).length > 0}
                    <span class="text-[10px] text-purple-500 font-mono shrink-0">
                      groups: {JSON.stringify(m.groups)}
                    </span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
        <div class="flex items-center gap-2">
          <span class="inline-block w-2 h-2 rounded-full {error ? 'bg-red-500' : matches.length > 0 ? 'bg-green-500' : 'bg-gray-400'}"></span>
          <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
            {error ? $_('regexTester__error') || 'Error' : `${matches.length} ${$_('regexTester__matches_found') || 'matches'}`}
          </span>
        </div>
        <div class="text-[10px] text-gray-400 font-mono">
          /{pattern || '...'}/{flags}
        </div>
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
