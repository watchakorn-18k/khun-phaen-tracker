<script lang="ts">
  import {
    Play,
    Pause,
    RotateCcw,
    Timer,
    HelpCircle,
    Save,
    History,
    Settings2,
    Bell,
    BellOff,
    X,
    Check,
    Clock,
    Zap,
    Target,
  } from "lucide-svelte";
  import { timerStore, formattedTime, timerProgress } from "$lib/stores/timerStore";
  import { showKeyboardShortcuts } from "$lib/stores/keyboardShortcuts";
  import { timeLogs, formatDuration } from "$lib/stores/timeLogs";
  import { _ } from "svelte-i18n";
  import { fade, slide, scale } from "svelte/transition";
  import { createUIActions } from "$lib/stores/uiActions";

  const ui = createUIActions();
  const pomodoroPresets = [15, 25, 45, 60, 90];

  let showSaveDialog = false;
  let saveNote = "";
  let showLogs = false;

  const modes = [
    { id: "countup", label: "timer__mode_countup", icon: Zap },
    { id: "pomodoro", label: "timer__mode_pomodoro", icon: Clock },
    { id: "countdown", label: "timer__mode_goal", icon: Target },
  ];

  function toggleTimer() {
    timerStore.toggle();
  }

  function resetTimer() {
    timerStore.reset();
  }

  function setMode(mode: any) {
    timerStore.setMode(mode);
  }

  function saveTime() {
    if ($timerStore.elapsed > 0) {
      timeLogs.add($timerStore.elapsed, saveNote);
      saveNote = "";
      showSaveDialog = false;
    }
  }

  function formatTime(totalSeconds: number): string {
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const hStr = hours > 0 ? `${hours}:` : "";
    const mStr = minutes.toString().padStart(hours > 0 ? 2 : 1, "0");
    const sStr = seconds.toString().padStart(2, "0");
    return `${hStr}${mStr}:${sStr}`;
  }

  function handleClose() {
    ui.closeModal("timerDashboard");
  }

  $: activeModeIndex = modes.findIndex(m => m.id === $timerStore.timerMode);
</script>

<div
  class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/40 backdrop-blur-sm"
  transition:fade
  on:mousedown|self={handleClose}
  role="presentation"
>
  <div
    class="glass-card w-full max-w-[340px] overflow-hidden shadow-2xl flex flex-col relative"
    transition:scale={{ duration: 250, start: 0.9, opacity: 0 }}
    on:mousedown|stopPropagation
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Background Decoration -->
    <div class="absolute top-0 right-0 w-32 h-32 bg-primary/10 rounded-full blur-3xl -mr-16 -mt-16 pointer-events-none"></div>

    <!-- Header -->
    <div class="p-5 flex items-center justify-between relative z-10">
      <div>
        <h2 class="text-xl font-black text-slate-800 dark:text-white tracking-tight">{$_("timer__title")}</h2>
        <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5">
           {$timerStore.isRunning ? $_("timer__status_active") : $_("timer__status_paused")}
        </p>
      </div>
      <div class="flex items-center gap-2">
         <button on:click={() => timerStore.updateSettings({ soundEnabled: !$timerStore.soundEnabled })} class="icon-btn" title="Toggle Sound">
          {#if $timerStore.soundEnabled}<Bell size={18} />{:else}<BellOff size={18} />{/if}
        </button>
        <button on:click={() => (showLogs = !showLogs)} class="icon-btn {showLogs ? 'bg-primary/10 text-primary' : ''}" title="History">
          <History size={18} />
        </button>
        <button on:click={handleClose} class="icon-btn hover:bg-red-500/10 hover:text-red-500 transition-colors">
          <X size={20} />
        </button>
      </div>
    </div>

    <!-- Mode Tabs -->
    <div class="px-5 pb-2">
      <div class="relative flex bg-slate-100 dark:bg-white/5 p-1 rounded-2xl border border-black/5 dark:border-white/5">
        <!-- Sliding Highlight -->
        <div 
          class="absolute top-1 bottom-1 left-1 bg-white dark:bg-slate-800 rounded-xl shadow-sm transition-all duration-300 ease-out z-0"
          style="width: calc(33.33% - 4px); transform: translateX({activeModeIndex * 100}%);"
        ></div>
        
        {#each modes as mode, i}
          <button
            on:click={() => setMode(mode.id)}
            class="relative z-10 flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-xs font-bold transition-colors {activeModeIndex === i ? 'text-primary' : 'text-slate-500 dark:text-white/40 hover:text-slate-700 dark:hover:text-white/60'}"
          >
            <svelte:component this={mode.icon} size={14} />
            <span>{$_(mode.label)}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Main Content -->
    <div class="p-5 pt-2 space-y-6">
      <!-- Time Display -->
      <div 
         class="relative group cursor-pointer" 
         on:click={toggleTimer}
         role="button"
         tabindex="0"
         on:keydown={(e) => e.key === 'Enter' && toggleTimer()}
      >
         <div class="text-center py-6 transition-transform group-active:scale-95">
            <div class="text-5xl font-mono font-black text-slate-800 dark:text-white tabular-nums tracking-tighter">
              {$formattedTime}
            </div>
            {#if $timerStore.timerMode !== 'countup'}
              <div class="mt-4 px-8">
                 <div class="w-full h-2 bg-slate-100 dark:bg-white/5 rounded-full overflow-hidden border border-black/5 dark:border-white/5">
                    <div class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 transition-all duration-500 ease-out shadow-[0_0_8px_rgba(99,102,241,0.5)]" style="width: {$timerProgress}%"></div>
                 </div>
                 <p class="text-[10px] font-bold text-slate-400 uppercase tracking-widest mt-2">
                   {$_("timer__progress_completed", { values: { percent: Math.round($timerProgress) } })}
                 </p>
              </div>
            {/if}
         </div>
      </div>

      <!-- Settings Panel -->
      <div class="min-h-[140px] bg-slate-50 dark:bg-white/5 rounded-2xl p-4 border border-black/5 dark:border-white/5 flex flex-col justify-center">
        {#if $timerStore.timerMode === "pomodoro"}
          <div transition:fade={{ duration: 150 }} class="space-y-4">
            <div class="flex items-center justify-between px-1">
               <span class="text-[10px] font-black text-slate-500 uppercase tracking-widest">{$_("timer__config_title")}</span>
               <div class="flex gap-1">
                 {#each pomodoroPresets as min}
                   <button
                     on:click={() => timerStore.updateSettings({ pomodoroWorkMinutes: min })}
                     class="w-7 h-7 flex items-center justify-center text-[10px] font-bold rounded-lg border transition-all {$timerStore.pomodoroWorkMinutes === min ? 'bg-primary border-primary text-white' : 'bg-white dark:bg-slate-800 border-black/10 text-slate-500'}"
                   >
                     {min}
                   </button>
                 {/each}
               </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label for="p-work" class="text-[10px] font-bold uppercase text-slate-400 ml-1">{$_("timer__config_focus")}</label>
                <div class="relative">
                   <input id="p-work" type="number" bind:value={$timerStore.pomodoroWorkMinutes} class="w-full bg-white dark:bg-slate-800 border border-black/10 dark:border-white/10 rounded-xl px-3 py-2 text-sm text-slate-800 dark:text-white font-bold focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
                   <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-slate-400">{$_("statsPanel__minutes_short").toUpperCase()}</span>
                </div>
              </div>
              <div class="space-y-1.5">
                <label for="p-break" class="text-[10px] font-bold uppercase text-slate-400 ml-1">{$_("timer__config_break")}</label>
                <div class="relative">
                   <input id="p-break" type="number" bind:value={$timerStore.pomodoroBreakMinutes} class="w-full bg-white dark:bg-slate-800 border border-black/10 dark:border-white/10 rounded-xl px-3 py-2 text-sm text-slate-800 dark:text-white font-bold focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
                   <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-slate-400">{$_("statsPanel__minutes_short").toUpperCase()}</span>
                </div>
              </div>
            </div>
          </div>
        {:else if $timerStore.timerMode === "countdown"}
          <div transition:fade={{ duration: 150 }} class="space-y-4">
            <span class="text-[10px] font-black text-slate-500 uppercase tracking-widest px-1">{$_("timer__target_title")}</span>
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label for="g-h" class="text-[10px] font-bold uppercase text-slate-400 ml-1">{$_("timer__settings_hours")}</label>
                <input id="g-h" type="number" bind:value={$timerStore.targetHours} class="w-full bg-white dark:bg-slate-800 border border-black/10 dark:border-white/10 rounded-xl px-3 py-2 text-sm text-slate-800 dark:text-white font-bold focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
              </div>
              <div class="space-y-1.5">
                <label for="g-m" class="text-[10px] font-bold uppercase text-slate-400 ml-1">{$_("timer__settings_minutes")}</label>
                <input id="g-m" type="number" bind:value={$timerStore.targetMinutes} class="w-full bg-white dark:bg-slate-800 border border-black/10 dark:border-white/10 rounded-xl px-3 py-2 text-sm text-slate-800 dark:text-white font-bold focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
              </div>
            </div>
          </div>
        {:else}
          <div transition:fade={{ duration: 150 }} class="text-center py-4 flex flex-col items-center gap-3">
            <div class="w-12 h-12 rounded-2xl bg-primary/10 flex items-center justify-center text-primary">
               <Zap size={24} />
            </div>
            <div class="space-y-1">
              <p class="text-sm font-bold text-slate-800 dark:text-white">{$_("timer__session_active_title")}</p>
              <p class="text-xs text-slate-400">{$_("timer__session_active_desc")}</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Action Footer -->
      <div class="flex gap-3 relative z-10">
        <button
          on:click={resetTimer}
          class="flex-1 py-3 px-4 bg-slate-100 dark:bg-white/5 hover:bg-slate-200 dark:hover:bg-white/10 text-slate-700 dark:text-white font-bold rounded-2xl transition-all flex items-center justify-center gap-2 border border-black/5 dark:border-white/5"
        >
          <RotateCcw size={16} />
          <span>{$_("timer__btn_reset")}</span>
        </button>
        
        {#if $timerStore.isRunning}
          <button
            on:click={toggleTimer}
            class="flex-1 py-3 px-4 bg-red-500 hover:bg-red-600 text-white font-bold rounded-2xl transition-all flex items-center justify-center gap-2 shadow-lg shadow-red-500/20"
          >
            <Pause size={18} fill="currentColor" />
            <span>{$_("timer__status_paused")}</span>
          </button>
        {:else}
          <button
            on:click={toggleTimer}
            class="flex-1 py-3 px-4 bg-primary hover:bg-primary-dark text-white font-bold rounded-2xl transition-all flex items-center justify-center gap-2 shadow-lg shadow-primary/20"
          >
            <Play size={18} fill="currentColor" />
            <span>{$_("timer__status_active")}</span>
          </button>
        {/if}

        {#if $timerStore.elapsed > 0}
          <button
            on:click={() => (showSaveDialog = true)}
            class="w-12 h-12 flex items-center justify-center bg-emerald-500 hover:bg-emerald-600 text-white rounded-2xl shadow-lg shadow-emerald-500/20 transition-all"
            title={$_("timer__btn_save")}
          >
            <Save size={20} />
          </button>
        {/if}
      </div>
    </div>

    <!-- Logs Popover Overlay -->
    {#if showLogs}
      <div 
        class="absolute inset-x-0 bottom-0 top-[110px] bg-white dark:bg-slate-900 z-[20] flex flex-col border-t border-black/5 dark:border-white/5"
        transition:slide={{ duration: 300 }}
      >
        <div class="p-4 border-b border-black/5 dark:border-white/5 flex items-center justify-between">
           <h3 class="text-xs font-black uppercase text-slate-500 tracking-widest">{$_("timer__history_recent_activity")}</h3>
           <button on:click={() => (showLogs = false)} class="p-1 hover:bg-slate-100 dark:hover:bg-white/5 rounded-lg"><X size={14} /></button>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar">
           {#each [...$timeLogs].reverse().slice(0, 10) as log}
            <div class="flex items-center justify-between p-3 rounded-2xl bg-slate-50 dark:bg-white/5 border border-black/5 group">
              <div class="flex flex-col gap-0.5">
                <span class="text-sm font-black text-primary">{formatDuration(log.duration)}</span>
                {#if log.note}<span class="text-[10px] text-slate-500 dark:text-white/40 italic truncate max-w-[180px]">{log.note}</span>{/if}
              </div>
              <button 
                on:click={() => timeLogs.remove(log.id)} 
                class="opacity-0 group-hover:opacity-100 p-2 text-slate-300 hover:text-red-500 transition-all"
              >
                <X size={14} />
              </button>
            </div>
           {/each}
           {#if $timeLogs.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-center opacity-30 py-8">
               <History size={32} class="mb-2" />
               <p class="text-xs font-bold uppercase tracking-widest">{$_("timer__history_empty_state")}</p>
            </div>
           {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Save Dialog -->
{#if showSaveDialog}
  <div class="fixed inset-0 z-[110] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm" transition:fade>
    <div class="glass-card w-full max-w-[320px] p-6 space-y-5 shadow-2xl" transition:scale={{ duration: 200, start: 0.9 }}>
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-black text-slate-800 dark:text-white tracking-tight">{$_("timer__dialog_save_title")}</h3>
          <p class="text-[10px] font-bold text-primary uppercase tracking-widest">{$_("timer__save_recording")}</p>
        </div>
        <div class="bg-primary/10 px-3 py-1.5 rounded-xl border border-primary/20">
           <span class="text-primary font-mono font-black text-lg">{formatTime($timerStore.elapsed)}</span>
        </div>
      </div>
      
      <div class="space-y-2">
        <label for="save-note" class="text-[10px] font-black uppercase text-slate-400 ml-1">{$_("timer__dialog_note_label")}</label>
        <textarea 
          id="save-note" 
          bind:value={saveNote} 
          placeholder={$_("timer__dialog_note_placeholder")} 
          class="w-full h-32 bg-slate-50 dark:bg-white/5 border border-black/10 dark:border-white/10 rounded-2xl p-4 text-sm text-slate-800 dark:text-white focus:ring-2 focus:ring-primary/20 outline-none transition-all resize-none"
        ></textarea>
      </div>
      
      <div class="flex gap-3">
        <button on:click={() => (showSaveDialog = false)} class="flex-1 py-3 px-4 bg-slate-100 dark:bg-white/5 text-slate-600 dark:text-white font-bold rounded-2xl border border-black/5">{$_("common.cancel")}</button>
        <button on:click={saveTime} class="flex-2 py-3 px-4 bg-primary hover:bg-primary-dark text-white font-bold rounded-2xl shadow-lg shadow-primary/20 flex items-center justify-center gap-2">
          <Check size={18} />
          <span>{$_("timer__btn_save_session")}</span>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .glass-card {
    background: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 2rem;
  }
  :global(.dark) .glass-card {
    background: rgba(15, 20, 30, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  
  .icon-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    color: #64748b;
    transition: all 0.2s;
  }
  :global(.dark) .icon-btn {
    color: #94a3b8;
  }
  .icon-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: #1e293b;
    transform: translateY(-1px);
  }
  :global(.dark) .icon-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: white;
  }
  
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 10px;
  }
  :global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
  }
  
  input[type="number"]::-webkit-inner-spin-button,
  input[type="number"]::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
</style>
