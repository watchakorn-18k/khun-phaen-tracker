<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { Code2, Eye, X, Copy, Check, Maximize2, Minimize2, Columns2 } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();

  const STORAGE_KEY = "html-preview-code";

  let code = $state("");
  let copied = $state(false);
  let isFullscreen = $state(false);
  let splitRatio = $state(50);
  let isDragging = $state(false);
  let containerEl: HTMLElement | null = null;

  onMount(() => {
    if (browser) {
      code = localStorage.getItem(STORAGE_KEY) || "";
      const saved = localStorage.getItem("html-preview-split");
      if (saved) splitRatio = Number(saved);
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleClose();
  }

  function handleClose() {
    dispatch("close");
  }

  function handleCodeChange(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    code = target.value;
    if (browser) {
      localStorage.setItem(STORAGE_KEY, code);
    }
  }

  function handleCopy() {
    navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function toggleFullscreen() {
    isFullscreen = !isFullscreen;
  }

  function handleDividerDrag(e: MouseEvent) {
    e.preventDefault();
    if (!containerEl) return;

    isDragging = true;
    const container = containerEl;

    const onMove = (moveEvent: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const ratio = ((moveEvent.clientX - rect.left) / rect.width) * 100;
      splitRatio = Math.min(80, Math.max(20, ratio));
    };

    const onUp = () => {
      isDragging = false;
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      if (browser) {
        localStorage.setItem("html-preview-split", String(splitRatio));
      }
    };

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }
</script>

{#if browser}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-xl shadow-xl {isFullscreen ? 'w-full h-full' : 'max-w-7xl w-full h-[85vh]'} flex flex-col animate-modal-in overflow-hidden transition-all duration-300"
      role="dialog"
      aria-modal="true"
      aria-labelledby="html-preview-title"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-orange-100 dark:bg-orange-900/30 rounded-lg">
            <Code2 class="text-orange-600 dark:text-orange-400" size={20} />
          </div>
          <div>
            <h3 id="html-preview-title" class="text-lg font-semibold text-gray-900 dark:text-white">
              {$_('htmlPreview__title') || 'HTML Preview'}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {$_('htmlPreview__subtitle') || 'Write HTML and see a live preview'}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={handleCopy}
            class="p-2 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all"
            title={$_('htmlPreview__copy') || 'Copy code'}
          >
            {#if copied}
              <Check size={18} class="text-green-500" />
            {:else}
              <Copy size={18} />
            {/if}
          </button>

          <button
            onclick={toggleFullscreen}
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all"
            title={isFullscreen ? $_('htmlPreview__exit_fullscreen') || 'Exit fullscreen' : $_('htmlPreview__fullscreen') || 'Fullscreen'}
          >
            {#if isFullscreen}
              <Minimize2 size={18} />
            {:else}
              <Maximize2 size={18} />
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

      <!-- Split Content -->
      <div class="flex-1 flex overflow-hidden split-container" bind:this={containerEl}>
        <!-- Drag overlay to block iframe from capturing mouse events -->
        {#if isDragging}
          <div class="fixed inset-0 z-[99999] cursor-col-resize"></div>
        {/if}
        <!-- Left: Code Editor -->
        <div class="flex flex-col overflow-hidden border-r border-gray-200 dark:border-gray-700" style="width: {splitRatio}%;">
          <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
            <Code2 size={14} class="text-orange-500" />
            <span class="text-[11px] font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              {$_('htmlPreview__tab_code') || 'Code'}
            </span>
          </div>
          <div class="flex-1 flex overflow-hidden">
            <div class="w-12 bg-gray-50 dark:bg-gray-900/50 border-r border-gray-200 dark:border-gray-700 pt-4 overflow-y-auto select-none shrink-0">
              {#each code.split('\n') as _, i}
                <div class="text-[11px] text-gray-400 dark:text-gray-600 text-right pr-3 leading-[1.625rem] font-mono">
                  {i + 1}
                </div>
              {/each}
            </div>
            <textarea
              value={code}
              oninput={handleCodeChange}
              class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none leading-[1.625rem] placeholder-gray-400 dark:placeholder-gray-600"
              placeholder="{$_('htmlPreview__placeholder') || 'Paste or type HTML here...'}"
              spellcheck="false"
            ></textarea>
          </div>
        </div>

        <!-- Divider -->
        <div
          class="w-1.5 bg-gray-200 dark:bg-gray-700 hover:bg-indigo-400 dark:hover:bg-indigo-500 cursor-col-resize transition-colors relative group shrink-0"
          role="separator"
          onmousedown={handleDividerDrag}
        >
          <div class="absolute inset-y-0 -left-1 -right-1"></div>
          <div class="absolute inset-y-0 left-1/2 -translate-x-1/2 w-0.5 bg-gray-400 dark:bg-gray-500 group-hover:bg-indigo-500 transition-colors"></div>
        </div>

        <!-- Right: Preview -->
        <div class="flex flex-col overflow-hidden" style="width: {100 - splitRatio}%;">
          <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
            <Eye size={14} class="text-indigo-500" />
            <span class="text-[11px] font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              {$_('htmlPreview__tab_preview') || 'Preview'}
            </span>
          </div>
          <div class="flex-1 overflow-hidden bg-white">
            <iframe
              srcdoc={code}
              title="HTML Preview"
              class="w-full h-full border-none"
              sandbox="allow-scripts"
            ></iframe>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
        <div class="flex items-center gap-2">
          <span class="inline-block w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
          <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">HTML</span>
        </div>
        <div class="text-[10px] text-gray-400">
          {code.length} {$_('htmlPreview__characters') || 'characters'}
          &middot;
          {code.split('\n').length} {$_('htmlPreview__lines') || 'lines'}
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

  textarea {
    tab-size: 2;
  }
</style>
