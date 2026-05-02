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
  class="w-64 overflow-hidden rounded-xl border border-gray-700/80 bg-gray-950 py-2 text-gray-100 shadow-2xl animate-in fade-in zoom-in duration-150 origin-top-left"
>
  <div class="mb-2 border-b border-gray-800 px-4 pb-2 pt-1 flex justify-between items-center">
    <h3 class="text-xs font-bold text-gray-400 uppercase tracking-wider">
      {$_("columnSettings__title", { default: "Card properties" })}
    </h3>
  </div>

  <div class="px-1 max-h-80 overflow-y-auto">
    {#each $columnSettings as setting}
      <button
        on:click={() => columnSettings.toggle(setting.id)}
        class="w-full flex items-center justify-between px-3 py-1.5 text-left text-sm transition-colors hover:bg-white/5 group"
      >
        <span class="font-medium text-gray-200 group-hover:text-white">
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

  <div class="mt-2 border-t border-gray-800 px-2 pt-2">
    <button
      on:click={() => columnSettings.reset()}
      class="w-full flex items-center justify-center gap-2 px-3 py-1.5 text-xs font-medium text-gray-400 transition-colors hover:bg-white/5 hover:text-white"
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
