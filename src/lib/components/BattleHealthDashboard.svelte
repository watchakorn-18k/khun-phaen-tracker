<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { Task } from "$lib/types";
  import {
    Sword,
    ShieldAlert,
    TrendingUp,
    TrendingDown,
    Zap,
    Skull,
    Minus,
    Flame,
    Users,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";

  import { calculateBattleHealthStats } from "../utils/battleHealth";

  export let tasks: Task[] = [];

  $: stats = calculateBattleHealthStats(tasks);

  function getMoraleColor(morale: string) {
    switch (morale) {
      case "excellent":
        return "from-emerald-500 to-teal-600 bg-emerald-50 content-emerald-600";
      case "stable":
        return "from-blue-500 to-indigo-600 bg-blue-50 content-blue-600";
      case "warning":
        return "from-amber-500 to-orange-600 bg-amber-50 content-amber-600";
      case "critical":
        return "from-rose-500 to-red-600 bg-rose-50 content-rose-600";
      default:
        return "from-gray-500 to-slate-600 bg-gray-50 content-gray-600";
    }
  }

  function getMoraleIcon(morale: string) {
    switch (morale) {
      case "excellent":
        return Flame;
      case "stable":
        return Sword;
      case "warning":
        return ShieldAlert;
      case "critical":
        return Skull;
      default:
        return Users;
    }
  }
</script>

<div class="space-y-4" in:fade={{ duration: 300 }}>
  <div class="flex items-center gap-2 mb-2">
    <div class="p-2 bg-primary/10 rounded-lg text-primary">
      <Sword size={20} />
    </div>
    <h2 class="text-lg font-bold text-gray-800 dark:text-white">
      {$_("workload__battle_dashboard")}
    </h2>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <!-- Velocity Card -->
    <div
      class="relative overflow-hidden group bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 shadow-lg p-5 transition-all hover:shadow-xl"
    >
      <div
        class="absolute -right-4 -top-4 opacity-[0.03] group-hover:opacity-[0.06] transition-opacity"
      >
        <TrendingUp size={120} />
      </div>

      <div class="flex flex-col h-full">
        <div class="flex items-center justify-between mb-4">
          <span
            class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400"
          >
            {$_("workload__project_velocity")}
          </span>
          <div
            class="p-2 bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 rounded-xl"
          >
            <Zap size={18} />
          </div>
        </div>

        <div class="flex items-end gap-2 mb-2">
          <span
            class="text-4xl font-black text-gray-900 dark:text-white leading-none"
          >
            {stats.velocity}
          </span>
          <span
            class="text-sm font-medium text-gray-500 dark:text-gray-400 pb-1"
          >
            tasks/day
          </span>
        </div>

        <div class="mt-auto pt-4 flex items-center gap-2">
          {#if stats.velocityTrend > 0}
            <div
              class="flex items-center text-emerald-600 dark:text-emerald-400 text-xs font-bold bg-emerald-50 dark:bg-emerald-900/20 px-2 py-1 rounded-full"
            >
              <TrendingUp size={12} class="mr-1" />
              +{stats.velocityTrend}
            </div>
            <span class="text-[10px] text-gray-400 uppercase tracking-tighter">
              {$_("workload__velocity_up")}
            </span>
          {:else if stats.velocityTrend < 0}
            <div
              class="flex items-center text-rose-600 dark:text-rose-400 text-xs font-bold bg-rose-50 dark:bg-rose-900/20 px-2 py-1 rounded-full"
            >
              <TrendingDown size={12} class="mr-1" />
              {stats.velocityTrend}
            </div>
            <span class="text-[10px] text-gray-400 uppercase tracking-tighter">
              {$_("workload__velocity_down")}
            </span>
          {:else}
            <div
              class="flex items-center text-gray-500 dark:text-gray-400 text-xs font-bold bg-gray-50 dark:bg-gray-700/50 px-2 py-1 rounded-full"
            >
              <Minus size={12} class="mr-1" />
              0
            </div>
            <span class="text-[10px] text-gray-400 uppercase tracking-tighter">
              {$_("workload__velocity_stable")}
            </span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Overdue Ratio Card -->
    <div
      class="relative overflow-hidden group bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 shadow-lg p-5 transition-all hover:shadow-xl"
    >
      <div
        class="absolute -right-4 -top-4 opacity-[0.03] group-hover:opacity-[0.06] transition-opacity"
      >
        <ShieldAlert size={120} />
      </div>

      <div class="flex flex-col h-full">
        <div class="flex items-center justify-between mb-4">
          <span
            class="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400"
          >
            {$_("workload__overdue_ratio")}
          </span>
          <div
            class="p-2 bg-rose-50 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400 rounded-xl"
          >
            <ShieldAlert size={18} />
          </div>
        </div>

        <div class="flex items-end gap-2 mb-2">
          <span
            class="text-4xl font-black text-gray-900 dark:text-white leading-none"
          >
            {stats.overdueRatio.toFixed(0)}<span class="text-2xl font-bold"
              >%</span
            >
          </span>
        </div>

        <div
          class="w-full bg-gray-100 dark:bg-gray-700 h-2 rounded-full mt-2 overflow-hidden"
        >
          <div
            class="h-full transition-all duration-500 ease-out {stats.overdueRatio >
            20
              ? 'bg-rose-500'
              : stats.overdueRatio > 10
                ? 'bg-amber-500'
                : 'bg-emerald-500'}"
            style="width: {Math.min(stats.overdueRatio, 100)}%"
          ></div>
        </div>

        <div class="mt-auto pt-4 flex items-center gap-1">
          <span
            class="text-xs font-bold {stats.overdueCount > 0
              ? 'text-rose-600 dark:text-rose-400'
              : 'text-emerald-600 dark:text-emerald-400'}"
          >
            {stats.overdueCount}
          </span>
          <span class="text-[10px] text-gray-400 uppercase tracking-tighter">
            {$_("workload__overdue_tasks_summary", {
              values: {
                overdue: stats.overdueCount,
                active: stats.activeTasksCount,
              },
            })}
          </span>
        </div>
      </div>
    </div>

    <!-- Army Morale Card -->
    <div
      class="relative overflow-hidden group rounded-2xl border border-gray-100 dark:border-gray-700 shadow-xl p-5 transition-all text-white bg-linear-to-br {getMoraleColor(
        stats.morale,
      )
        .split(' ')
        .slice(0, 2)
        .join(' ')}"
    >
      <div
        class="absolute -right-4 -top-4 opacity-[0.15] group-hover:opacity-[0.2] transition-opacity"
      >
        <svelte:component this={getMoraleIcon(stats.morale)} size={120} />
      </div>

      <div class="flex flex-col h-full relative z-10">
        <div class="flex items-center justify-between mb-4">
          <span class="text-xs font-bold uppercase tracking-wider opacity-80">
            {$_("workload__army_morale")}
          </span>
          <div class="p-2 bg-white/20 backdrop-blur-sm rounded-xl">
            <svelte:component this={getMoraleIcon(stats.morale)} size={18} />
          </div>
        </div>

        <div class="mb-2">
          <span class="text-2xl font-black leading-tight drop-shadow-md">
            {$_(`workload__morale_${stats.morale}`)}
          </span>
        </div>

        <div class="mt-auto pt-2">
          <p
            class="text-[10px] font-medium opacity-80 uppercase tracking-wider"
          >
            {$_("workload__status_report")}
          </p>
          <div class="flex gap-2 mt-1">
            <div
              class="px-2 py-0.5 bg-white/20 backdrop-blur-md rounded-md text-[9px] font-bold"
            >
              +{stats.doneTasksLast7Days}
              {$_("workload__cleared_unit")}
            </div>
            <div
              class="px-2 py-0.5 bg-white/20 backdrop-blur-md rounded-md text-[9px] font-bold"
            >
              {stats.overdueCount}
              {$_("workload__casualties_unit")}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* Glassmorphism effects */
  :global(.dark) .group {
    background: rgba(31, 41, 55, 0.8);
    backdrop-filter: blur(10px);
  }
</style>
