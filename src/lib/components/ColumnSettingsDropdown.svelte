<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from "svelte-i18n";
  import { RotateCcw } from "lucide-svelte";
  import { columnSettings } from "$lib/stores/columnSettings";

  const dispatch = createEventDispatcher();

  function close() {
    dispatch("close");
  }
</script>

<div
  class="absolute top-full left-0 mt-2 w-64 bg-white dark:bg-gray-900 rounded-2xl shadow-xl border border-gray-200 dark:border-gray-800 py-2 z-50 animate-in fade-in zoom-in duration-200 origin-top-left"
>
  <div class="px-4 py-2 border-b border-gray-100 dark:border-gray-800 mb-1 flex justify-between items-center">
    <h3 class="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
      {$_("columnSettings__title", { default: "Card properties" })}
    </h3>
  </div>

  <div class="px-1 max-h-80 overflow-y-auto">
    {#each $columnSettings as setting}
      <button
        on:click={() => columnSettings.toggle(setting.id)}
        class="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors group"
      >
        <span class="text-sm font-medium text-gray-700 dark:text-gray-300 group-hover:text-gray-900 dark:group-hover:text-white">
          {$_(`columnSettings__label_${setting.id}`, { default: setting.label })}
        </span>
        
        <div
          class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {setting.enabled ? 'bg-primary' : 'bg-gray-200 dark:bg-gray-700'}"
        >
          <span
            aria-hidden="true"
            class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {setting.enabled ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </div>
      </button>
    {/each}
  </div>

  <div class="mt-2 pt-2 px-2 border-t border-gray-100 dark:border-gray-800">
    <button
      on:click={() => columnSettings.reset()}
      class="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium text-gray-500 hover:text-primary dark:text-gray-400 dark:hover:text-primary hover:bg-gray-50 dark:hover:bg-gray-800 rounded-xl transition-all"
    >
      <RotateCcw size={14} />
      {$_("columnSettings__reset", { default: "Reset to defaults" })}
    </button>
  </div>
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes zoomIn {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
  .animate-in {
    animation: fadeIn 0.2s ease-out, zoomIn 0.2s ease-out;
    animation-fill-mode: forwards;
  }
</style>
