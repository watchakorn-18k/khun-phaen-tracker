<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { GitCompare, X, Copy, Check, ArrowRightLeft, Info } from "lucide-svelte";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();

  let leftText = $state("");
  let rightText = $state("");
  let copied = $state(false);
  let showHelp = $state(false);

  type DiffLine = { type: "same" | "add" | "remove" | "change"; left: string; right: string; lineNum: number };
  let diffResult = $derived.by(() => {
    if (!leftText && !rightText) return [];
    const leftLines = leftText.split("\n");
    const rightLines = rightText.split("\n");
    const maxLen = Math.max(leftLines.length, rightLines.length);
    const result: DiffLine[] = [];

    // Simple LCS-based diff
    const lcs = buildLCS(leftLines, rightLines);
    let li = 0, ri = 0, oi = 0;

    while (oi < lcs.length || li < leftLines.length || ri < rightLines.length) {
      const lcsLine = oi < lcs.length ? lcs[oi] : null;
      const leftLine = li < leftLines.length ? leftLines[li] : null;
      const rightLine = ri < rightLines.length ? rightLines[ri] : null;

      if (lcsLine !== null && leftLine === lcsLine && rightLine === lcsLine) {
        result.push({ type: "same", left: leftLine, right: rightLine, lineNum: li + 1 });
        li++; ri++; oi++;
      } else if (lcsLine !== null && rightLine === lcsLine) {
        result.push({ type: "remove", left: leftLine || "", right: "", lineNum: li + 1 });
        li++;
      } else if (lcsLine !== null && leftLine === lcsLine) {
        result.push({ type: "add", left: "", right: rightLine || "", lineNum: ri + 1 });
        ri++;
      } else if (leftLine !== null && rightLine !== null && leftLine !== rightLine) {
        result.push({ type: "change", left: leftLine, right: rightLine, lineNum: li + 1 });
        li++; ri++;
      } else if (leftLine !== null) {
        result.push({ type: "remove", left: leftLine, right: "", lineNum: li + 1 });
        li++;
      } else if (rightLine !== null) {
        result.push({ type: "add", left: "", right: rightLine, lineNum: ri + 1 });
        ri++;
      } else {
        break;
      }
    }
    return result;
  });

  function buildLCS(a: string[], b: string[]): string[] {
    const m = a.length, n = b.length;
    // For performance, use simple LCS with limited size
    const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));
    for (let i = 1; i <= m; i++) {
      for (let j = 1; j <= n; j++) {
        dp[i][j] = a[i - 1] === b[j - 1] ? dp[i - 1][j - 1] + 1 : Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
    // Backtrack
    const result: string[] = [];
    let i = m, j = n;
    while (i > 0 && j > 0) {
      if (a[i - 1] === b[j - 1]) { result.unshift(a[i - 1]); i--; j--; }
      else if (dp[i - 1][j] > dp[i][j - 1]) i--;
      else j--;
    }
    return result;
  }

  let stats = $derived.by(() => {
    let added = 0, removed = 0, changed = 0;
    for (const d of diffResult) {
      if (d.type === "add") added++;
      else if (d.type === "remove") removed++;
      else if (d.type === "change") changed++;
    }
    return { added, removed, changed, same: diffResult.length - added - removed - changed };
  });

  let leftLabel = $state("Left");
  let rightLabel = $state("Right");

  function handleKeydown(e: KeyboardEvent) { if (e.key === "Escape") handleClose(); }
  function handleClose() { dispatch("close"); }

  function handleCopy() {
    const text = diffResult.map((d) => {
      if (d.type === "add") return `+ ${d.right}`;
      if (d.type === "remove") return `- ${d.left}`;
      if (d.type === "change") return `- ${d.left}\n+ ${d.right}`;
      return `  ${d.left}`;
    }).join("\n");
    navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function swapTexts() {
    const tmp = leftText; leftText = rightText; rightText = tmp;
    const tl = leftLabel; leftLabel = rightLabel; rightLabel = tl;
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  onMount(() => {
    if (!browser) return;
    const s = localStorage.getItem("diff-left"); if (s) leftText = s;
    const r = localStorage.getItem("diff-right"); if (r) rightText = r;
  });

  $effect(() => { if (browser) localStorage.setItem("diff-left", leftText); });
  $effect(() => { if (browser) localStorage.setItem("diff-right", rightText); });
</script>

{#if browser}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
    role="presentation" onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}>
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-6xl w-full h-[85vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog" aria-modal="true" tabindex="-1" onkeydown={handleKeydown}>

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-rose-100 dark:bg-rose-900/30 rounded-lg">
            <GitCompare class="text-rose-600 dark:text-rose-400" size={20} />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Diff Checker</h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">เทียบข้อความ/JSON 2 ชุด</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button onclick={() => showHelp = !showHelp} class="p-2 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all" title="How to use">
            <Info size={18} />
          </button>
          <button onclick={handleCopy} class="p-2 text-gray-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all" title="Copy diff">
            {#if copied}<Check size={18} class="text-green-500" />{:else}<Copy size={18} />{/if}
          </button>
          <button onclick={handleClose} class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
            <X size={20} />
          </button>
        </div>
      </div>

      <!-- Help Panel -->
      {#if showHelp}
        <div class="px-5 py-3 border-b border-gray-200 dark:border-gray-700 bg-indigo-50/50 dark:bg-indigo-900/10 text-[12px] leading-relaxed text-gray-600 dark:text-gray-300 space-y-2 shrink-0">
          <p class="font-bold text-indigo-600 dark:text-indigo-400">วิธีใช้งาน Diff Checker</p>
          <ul class="list-disc pl-4 space-y-1">
            <li>วางข้อความหรือ JSON ชุดแรกที่ช่อง <b>Left</b> และชุดที่สองที่ช่อง <b>Right</b></li>
            <li>ผลลัพธ์จะแสดงด้านล่างอัตโนมัติ — <span class="text-green-600 font-bold">เขียว</span> = เพิ่ม, <span class="text-red-600 font-bold">แดง</span> = ลบ, <span class="text-amber-600 font-bold">เหลือง</span> = เปลี่ยนแปลง</li>
            <li>แก้ Label ได้ เช่น Expected / Actual หรือ Staging / Prod</li>
            <li>กดปุ่ม <b>Swap</b> (สลับลูกศร) เพื่อสลับข้อความสองชุด</li>
            <li>กดปุ่ม <b>Copy</b> เพื่อคัดลอกผล diff เป็น unified format (+/-)</li>
            <li>ข้อมูลจะถูกบันทึกอัตโนมัติไว้ในเบราว์เซอร์</li>
          </ul>
        </div>
      {/if}

      <!-- Stats bar -->
      <div class="flex items-center gap-4 px-5 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
        <div class="flex items-center gap-1.5">
          <span class="inline-block w-2.5 h-2.5 rounded-sm bg-green-400"></span>
          <span class="text-[10px] font-bold text-gray-500">+{stats.added} added</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="inline-block w-2.5 h-2.5 rounded-sm bg-red-400"></span>
          <span class="text-[10px] font-bold text-gray-500">-{stats.removed} removed</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="inline-block w-2.5 h-2.5 rounded-sm bg-amber-400"></span>
          <span class="text-[10px] font-bold text-gray-500">~{stats.changed} changed</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="inline-block w-2.5 h-2.5 rounded-sm bg-gray-300 dark:bg-gray-600"></span>
          <span class="text-[10px] font-bold text-gray-500">{stats.same} same</span>
        </div>
        <div class="flex-1"></div>
        <button onclick={swapTexts} class="p-1 rounded text-gray-400 hover:text-rose-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-all" title="Swap">
          <ArrowRightLeft size={14} />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left Input -->
        <div class="flex flex-col overflow-hidden flex-1 border-r border-gray-200 dark:border-gray-700">
          <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
            <span class="text-[11px] font-bold text-red-500 uppercase tracking-wider">{leftLabel}</span>
            <input type="text" bind:value={leftLabel}
              class="flex-1 bg-transparent text-[10px] text-gray-400 outline-none" placeholder="Label..." />
          </div>
          <textarea bind:value={leftText}
            class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none placeholder-gray-400"
            placeholder="วางข้อความชุดแรกที่นี่..." spellcheck="false"></textarea>
        </div>

        <!-- Right Input -->
        <div class="flex flex-col overflow-hidden flex-1">
          <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
            <span class="text-[11px] font-bold text-green-500 uppercase tracking-wider">{rightLabel}</span>
            <input type="text" bind:value={rightLabel}
              class="flex-1 bg-transparent text-[10px] text-gray-400 outline-none" placeholder="Label..." />
          </div>
          <textarea bind:value={rightText}
            class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none placeholder-gray-400"
            placeholder="วางข้อความชุดที่สองที่นี่..." spellcheck="false"></textarea>
        </div>
      </div>

      <!-- Diff Result -->
      {#if diffResult.length > 0 && (leftText || rightText)}
        <div class="border-t border-gray-200 dark:border-gray-700 max-h-72 overflow-y-auto bg-gray-50 dark:bg-gray-900/30">
          <div class="px-4 py-2 sticky top-0 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
            <span class="text-[10px] font-bold text-gray-400 uppercase tracking-wider">Diff Result</span>
          </div>
          <div class="font-mono text-[12px] leading-6">
            {#each diffResult as d, i}
              {#if d.type === "same"}
                <div class="flex px-4 text-gray-400 dark:text-gray-500">
                  <span class="w-8 text-right pr-2 text-gray-300 dark:text-gray-600 shrink-0 select-none">{d.lineNum}</span>
                  <span class="w-4 shrink-0 select-none text-gray-300"> </span>
                  <span class="whitespace-pre-wrap break-all">{escapeHtml(d.left)}</span>
                </div>
              {:else if d.type === "remove"}
                <div class="flex px-4 bg-red-50 dark:bg-red-900/15 text-red-700 dark:text-red-300">
                  <span class="w-8 text-right pr-2 text-red-300 dark:text-red-600 shrink-0 select-none">{d.lineNum}</span>
                  <span class="w-4 shrink-0 select-none text-red-400 font-bold">-</span>
                  <span class="whitespace-pre-wrap break-all">{escapeHtml(d.left)}</span>
                </div>
              {:else if d.type === "add"}
                <div class="flex px-4 bg-green-50 dark:bg-green-900/15 text-green-700 dark:text-green-300">
                  <span class="w-8 text-right pr-2 text-green-300 dark:text-green-600 shrink-0 select-none">{d.lineNum}</span>
                  <span class="w-4 shrink-0 select-none text-green-400 font-bold">+</span>
                  <span class="whitespace-pre-wrap break-all">{escapeHtml(d.right)}</span>
                </div>
              {:else if d.type === "change"}
                <div class="flex px-4 bg-red-50 dark:bg-red-900/15 text-red-700 dark:text-red-300">
                  <span class="w-8 text-right pr-2 text-red-300 dark:text-red-600 shrink-0 select-none">{d.lineNum}</span>
                  <span class="w-4 shrink-0 select-none text-red-400 font-bold">-</span>
                  <span class="whitespace-pre-wrap break-all">{escapeHtml(d.left)}</span>
                </div>
                <div class="flex px-4 bg-green-50 dark:bg-green-900/15 text-green-700 dark:text-green-300">
                  <span class="w-8 text-right pr-2 text-green-300 dark:text-green-600 shrink-0 select-none"></span>
                  <span class="w-4 shrink-0 select-none text-green-400 font-bold">+</span>
                  <span class="whitespace-pre-wrap break-all">{escapeHtml(d.right)}</span>
                </div>
              {/if}
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  @keyframes modal-in { from { opacity: 0; transform: scale(0.95) translateY(-10px); } to { opacity: 1; transform: scale(1) translateY(0); } }
  .animate-modal-in { animation: modal-in 0.2s ease-out; }
  textarea { tab-size: 2; }
</style>
