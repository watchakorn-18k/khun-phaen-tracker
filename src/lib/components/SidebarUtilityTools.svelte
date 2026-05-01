<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    FileText,
    Bookmark,
    PenTool,
    Timer,
    Play,
    Pause,
    Save,
    Maximize2,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { timerStore, formattedTime } from "$lib/stores/timerStore";
  import { fade, slide } from "svelte/transition";
  import { createUIActions } from "$lib/stores/uiActions";

  const ui = createUIActions();
  const dispatch = createEventDispatcher<{
    showBookmarks: void;
    showWhiteboard: void;
    showQuickNotes: void;
  }>();

  export let isCollapsed = false;

  function toggleTimer() {
    timerStore.toggle();
  }

  function openDashboard() {
    ui.openModal("timerDashboard");
  }

  function handleQuickNotes() {
    ui.openModal("quickNotes");
  }

  function handleBookmarks() {
    ui.openModal("bookmarkManager");
  }

  function handleWhiteboard() {
    ui.openModal("whiteboard");
  }

  $: isRunning = $timerStore.isRunning;
  $: hasTime = $timerStore.elapsed > 0;
</script>

<div class="space-y-4">
  <!-- Utility Buttons Group -->
  <div>
    <div class="px-2 py-1 text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
      {$_("commandPalette__cat_commands")}
    </div>
    <div class="mt-1 space-y-0.5">
      <button
        on:click={handleQuickNotes}
        class="w-full flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white transition-colors"
      >
        <FileText size={16} class="shrink-0" />
        {#if !isCollapsed}
          <span>{$_("quickNotes__title")}</span>
        {/if}
      </button>

      <button
        on:click={handleBookmarks}
        class="w-full flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white transition-colors"
      >
        <Bookmark size={16} class="shrink-0" />
        {#if !isCollapsed}
          <span>{$_("timer__bookmarks_tooltip")}</span>
        {/if}
      </button>

      <button
        on:click={handleWhiteboard}
        class="w-full flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white transition-colors"
      >
        <PenTool size={16} class="shrink-0" />
        {#if !isCollapsed}
          <span>{$_("commandPalette__whiteboard_label")}</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Timer Widget in Sidebar -->
  <div class="px-2 group w-[75%] hover:w-full transition-all duration-500 ease-in-out">
    <div class="flex items-center justify-between py-1 mb-0.5 transition-all opacity-0 h-0 group-hover:h-auto group-hover:opacity-100 overflow-hidden">
      <div class="text-[10px] font-black text-indigo-500/70 dark:text-indigo-400/70 uppercase tracking-widest truncate max-w-[80px]">
        {$timerStore.timerMode === 'countup' ? $_('timer__mode_countup') : $timerStore.timerMode === 'pomodoro' ? $_('timer__mode_pomodoro') : $_('timer__mode_goal')}
      </div>
      <button 
        on:click={openDashboard}
        class="text-gray-400 hover:text-indigo-500 transition-all scale-75 group-hover:scale-100"
        title="Open Dashboard"
      >
        <Maximize2 size={12} />
      </button>
    </div>
    
    <div class="bg-gray-50/50 dark:bg-gray-800/20 border {isRunning ? 'border-red-500/60 ring-1 ring-red-500/20 shadow-[0_0_12px_rgba(239,68,68,0.15)]' : 'border-gray-200/60 dark:border-gray-700/30'} rounded-xl p-1 group-hover:p-3 transition-all duration-500 shadow-sm relative overflow-hidden group-hover:bg-white dark:group-hover:bg-gray-800 group-hover:border-indigo-500/30">
      <div class="flex items-center justify-between gap-1 group-hover:gap-2">
        <div class="flex flex-col">
          <span class="font-mono text-xs group-hover:text-lg font-black text-gray-800 dark:text-gray-200 leading-none transition-all duration-500 tracking-tighter group-hover:tracking-normal">
            {$formattedTime}
          </span>
          <span class="text-[0px] group-hover:text-[9px] text-gray-400 dark:text-gray-500 h-0 group-hover:h-auto mt-0 group-hover:mt-1 uppercase tracking-widest font-bold group-hover:text-gray-500 transition-all duration-500 truncate overflow-hidden">
            {$timerStore.timerMode === 'pomodoro' ? $timerStore.pomodoroPhase : $_("timer__status_active")}
          </span>
        </div>
        
        <div class="flex items-center gap-1 transition-all duration-500 {isRunning ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}">
          <button
            on:click={toggleTimer}
            class="w-6 h-6 group-hover:w-9 group-hover:h-9 flex items-center justify-center rounded-full transition-all duration-500 {isRunning
              ? 'bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/30'
              : 'bg-indigo-600 hover:bg-indigo-700 text-white shadow-lg shadow-indigo-600/30'}"
          >
            {#if isRunning}
              <Pause size={isRunning ? 10 : 14} class="group-hover:scale-125 transition-transform" fill="currentColor" />
            {:else}
              <Play size={isRunning ? 10 : 14} class="group-hover:scale-125 transition-transform ml-0.5" fill="currentColor" />
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- Quick Mode Switcher icons - ONLY visible on hover -->
    <div class="mt-2 flex items-center gap-2 justify-center opacity-0 h-0 group-hover:h-auto group-hover:opacity-100 transition-all duration-500 transform translate-y-1 group-hover:translate-y-0 pointer-events-none group-hover:pointer-events-auto overflow-hidden">
       <button 
         on:click={() => timerStore.setMode('countup')}
         class="p-1.5 rounded-lg transition-all {$timerStore.timerMode === 'countup' ? 'text-indigo-500 bg-indigo-500/10' : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
         title={$_('timer__mode_countup')}
       >
         <Timer size={14} />
       </button>
       <button 
         on:click={() => timerStore.setMode('pomodoro')}
         class="p-1.5 rounded-lg transition-all {$timerStore.timerMode === 'pomodoro' ? 'text-indigo-500 bg-indigo-500/10' : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
         title={$_('timer__mode_pomodoro')}
       >
         <Timer size={14} class="rotate-45" />
       </button>
       <button 
         on:click={() => timerStore.setMode('countdown')}
         class="p-1.5 rounded-lg transition-all {$timerStore.timerMode === 'countdown' ? 'text-indigo-500 bg-indigo-500/10' : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
         title={$_('timer__mode_goal')}
       >
         <Timer size={14} class="-rotate-45" />
       </button>
    </div>
  </div>
</div>

<style>
  button {
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    outline: none;
  }
</style>
