<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount, onDestroy } from "svelte";
  import type { Milestone } from "$lib/types/milestone";
  import { fade } from "svelte/transition";
  import { Pencil, Rocket, EyeOff, Trophy, CheckCircle } from "lucide-svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { api } from "$lib/apis";
  import { showMessage } from "$lib/stores/uiActions";

  let {
    milestone,
    onEdit,
    onDelete,
    onHide,
    isOwner = false,
  } = $props<{
    milestone: Milestone;
    onEdit?: (m: Milestone) => void;
    onDelete?: (m: Milestone) => void;
    onHide?: (m: Milestone) => void;
    isOwner?: boolean;
  }>();

  let showHideConfirm = $state(false);
  let isHiding = $state(false);

  async function handleToggleHide() {
    isHiding = true;
    try {
      const res = await api.workspaces.updateMilestone(
        milestone.workspace_id,
        milestone.id,
        {
          is_hidden: true,
        },
      );
      if (res.ok) {
        showMessage($_("page__success"));
        onHide?.(milestone);
      }
    } catch (e) {
      showMessage($_("page__error"), "error");
    } finally {
      isHiding = false;
      showHideConfirm = false;
    }
  }

  let timeLeft = $state({
    days: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
    isOver: false,
  });

  // Elapsed time since milestone was reached
  let elapsed = $state({
    days: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
  });

  const formatThaiDate = (dateStr: string) => {
    const months = [
      "ม.ค.",
      "ก.พ.",
      "มี.ค.",
      "เม.ย.",
      "พ.ค.",
      "มิ.ย.",
      "ก.ค.",
      "ส.ค.",
      "ก.ย.",
      "ต.ค.",
      "พ.ย.",
      "ธ.ค.",
    ];
    const date = new Date(dateStr);
    const day = date.getDate();
    const month = months[date.getMonth()];
    const year = date.getFullYear() + 543; // Buddhist Era
    const time = date.toLocaleTimeString("th-TH", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
    return `${day} ${month} ${year} (${time} น.)`;
  };

  function calculateTime() {
    const target = new Date(milestone.target_date).getTime();
    const now = new Date().getTime();
    const diff = target - now;

    if (diff <= 0) {
      timeLeft = { days: 0, hours: 0, minutes: 0, seconds: 0, isOver: true };

      // Calculate elapsed time since target
      const elapsedMs = Math.abs(diff);
      elapsed = {
        days: Math.floor(elapsedMs / (1000 * 60 * 60 * 24)),
        hours: Math.floor(
          (elapsedMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
        ),
        minutes: Math.floor((elapsedMs % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((elapsedMs % (1000 * 60)) / 1000),
      };
      return;
    }

    timeLeft = {
      days: Math.floor(diff / (1000 * 60 * 60 * 24)),
      hours: Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)),
      minutes: Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60)),
      seconds: Math.floor((diff % (1000 * 60)) / 1000),
      isOver: false,
    };
  }

  // Near launch = less than 24 hours remaining
  let isNearLaunch = $derived(
    !timeLeft.isOver &&
      timeLeft.days === 0 &&
      (timeLeft.hours > 0 || timeLeft.minutes > 0 || timeLeft.seconds > 0),
  );

  let interval: any;
  onMount(() => {
    calculateTime();
    interval = setInterval(calculateTime, 1000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<div
  class="relative overflow-hidden p-6 rounded-3xl shadow-2xl group transition-all hover:scale-[1.005] border border-white/20 {timeLeft.isOver
    ? 'bg-linear-to-br from-emerald-600 via-teal-600 to-cyan-700 shadow-emerald-500/30'
    : 'bg-linear-to-br from-indigo-600 via-indigo-700 to-purple-800 shadow-indigo-500/30'}"
  in:fade
>
  <!-- Shimmer overlay for completed state -->
  {#if timeLeft.isOver}
    <div class="absolute inset-0 animate-shimmer rounded-3xl z-0"></div>
  {/if}

  <!-- Animated Background Glow -->
  <div
    class="absolute -top-24 -left-24 w-64 h-64 blur-[80px] rounded-full animate-pulse {timeLeft.isOver
      ? 'bg-emerald-300/15'
      : 'bg-white/10'}"
  ></div>
  <div
    class="absolute -bottom-24 -right-24 w-64 h-64 blur-[80px] rounded-full animate-pulse {timeLeft.isOver
      ? 'bg-cyan-300/15'
      : 'bg-purple-400/20'}"
  ></div>

  <div class="relative z-10 flex flex-col xl:flex-row gap-8 items-center">
    <!-- Header/Title -->
    <div class="flex-1 min-w-0 text-center xl:text-left">
      <div class="flex items-center justify-center xl:justify-start gap-3 mb-3">
        {#if timeLeft.isOver}
          <!-- Trophy icon for completed -->
          <div
            class="p-2.5 bg-amber-400/20 backdrop-blur-md rounded-2xl border border-amber-300/40 animate-gentle-float shadow-lg shadow-amber-500/20"
          >
            <Trophy class="w-6 h-6 text-amber-300" />
          </div>
        {:else}
          <!-- Rocket icon for active countdown -->
          <div
            class="p-2.5 bg-white/20 backdrop-blur-md rounded-2xl border border-white/30 transition-all {isNearLaunch
              ? 'animate-rocket-launch bg-orange-500/30 border-orange-300/40 shadow-lg shadow-orange-500/20'
              : 'animate-gentle-float'}"
          >
            <Rocket class="w-6 h-6 text-white" />
          </div>
        {/if}
        <div>
          <h3
            class="text-2xl md:text-3xl font-black text-white truncate tracking-tight uppercase leading-none drop-shadow-lg flex items-center gap-2.5"
          >
            {milestone.title}
            {#if timeLeft.isOver}
              <!-- ✅ DONE badge -->
              <span
                class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-400/20 border border-emerald-300/40 text-[10px] font-black uppercase tracking-widest text-emerald-200 shrink-0"
              >
                <CheckCircle class="w-3 h-3" />
                DONE
              </span>
            {:else}
              <!-- 🔴 LIVE badge -->
              <span
                class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-red-500/20 border border-red-400/30 text-[10px] font-black uppercase tracking-widest text-red-300 shrink-0"
              >
                <span class="relative flex h-2 w-2">
                  <span
                    class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"
                  ></span>
                  <span
                    class="relative inline-flex rounded-full h-2 w-2 bg-red-500"
                  ></span>
                </span>
                LIVE
              </span>
            {/if}
          </h3>
          <p class="mt-1 text-sm text-white/70 font-medium tracking-wide">
            {formatThaiDate(milestone.target_date)}
          </p>
        </div>
      </div>
      <p
        class="text-sm md:text-base text-white/80 line-clamp-2 italic font-medium max-w-xl"
      >
        {milestone.description || $_("milestone.no_description")}
      </p>

      {#if isOwner}
        <div
          class="mt-5 flex items-center justify-center xl:justify-start gap-3 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <button
            onclick={() => onEdit?.(milestone)}
            class="p-2.5 bg-white/10 hover:bg-white/20 backdrop-blur-md rounded-xl transition-all border border-white/20 hover:border-white/40 group/btn"
            title={$_("milestone.edit_tooltip")}
          >
            <Pencil
              size={18}
              class="text-white group-hover/btn:scale-110 transition-transform"
            />
          </button>
          <button
            onclick={() => (showHideConfirm = true)}
            class="p-2.5 bg-white/10 hover:bg-white/20 backdrop-blur-md rounded-xl transition-all border border-white/20 hover:border-white/40 group/btn"
            title={$_("milestone.hide_tooltip")}
          >
            <EyeOff
              size={18}
              class="text-white group-hover/btn:scale-110 transition-transform"
            />
          </button>
        </div>
      {/if}
    </div>

    <!-- Countdown / Elapsed Units -->
    <div
      class="flex items-center gap-3 md:gap-6 shrink-0 backdrop-blur-lg p-5 md:p-7 rounded-[2.5rem] border shadow-2xl {timeLeft.isOver
        ? 'bg-white/10 border-white/15'
        : 'bg-black/20 border-white/10'}"
    >
      {#if timeLeft.isOver}
        <!-- Elapsed time display -->
        {#each [{ label: "days", value: elapsed.days, color: "from-emerald-400 to-teal-400" }, { label: "hours", value: elapsed.hours, color: "from-teal-400 to-cyan-400" }, { label: "mins", value: elapsed.minutes, color: "from-cyan-400 to-sky-400" }, { label: "secs", value: elapsed.seconds, color: "from-sky-400 to-blue-400" }] as unit}
          <div class="flex flex-col items-center">
            <div
              class="w-16 h-16 md:w-24 md:h-24 flex items-center justify-center bg-linear-to-b from-white/15 to-white/5 rounded-3xl border border-emerald-300/20 shadow-xl relative group/unit overflow-hidden"
            >
              <div
                class="absolute inset-0 bg-linear-to-br {unit.color} opacity-10"
              ></div>
              <span
                class="text-3xl md:text-5xl font-black text-white tabular-nums drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]"
              >
                {String(unit.value).padStart(2, "0")}
              </span>
            </div>
            <span
              class="mt-3 text-[10px] md:text-xs font-black uppercase tracking-[0.2em] text-white/50"
            >
              {$_(`milestone.${unit.label}`)}
            </span>
          </div>

          {#if unit.label !== "secs"}
            <div class="mb-8 text-white/40 font-black text-2xl md:text-4xl">
              :
            </div>
          {/if}
        {/each}
      {:else}
        <!-- Active countdown display -->
        {#each [{ label: "days", value: timeLeft.days, color: "from-blue-400 to-indigo-400" }, { label: "hours", value: timeLeft.hours, color: "from-indigo-400 to-purple-400" }, { label: "mins", value: timeLeft.minutes, color: "from-purple-400 to-pink-400" }, { label: "secs", value: timeLeft.seconds, color: "from-pink-400 to-rose-400" }] as unit}
          <div class="flex flex-col items-center">
            <div
              class="w-16 h-16 md:w-24 md:h-24 flex items-center justify-center bg-linear-to-b from-white/10 to-white/5 rounded-3xl border border-white/20 shadow-xl relative group/unit overflow-hidden"
            >
              <div
                class="absolute inset-0 bg-linear-to-br {unit.color} opacity-0 group-hover/unit:opacity-20 transition-opacity"
              ></div>
              <span
                class="text-3xl md:text-5xl font-black text-white tabular-nums drop-shadow-[0_0_15px_rgba(255,255,255,0.4)]"
              >
                {String(unit.value).padStart(2, "0")}
              </span>
            </div>
            <span
              class="mt-3 text-[10px] md:text-xs font-black uppercase tracking-[0.2em] text-white/50"
            >
              {$_(`milestone.${unit.label}`)}
            </span>
          </div>

          {#if unit.label !== "secs"}
            <div class="mb-8 text-white/40 font-black text-2xl md:text-4xl">
              :
            </div>
          {/if}
        {/each}
      {/if}
    </div>

    <!-- Status Badge -->
    {#if timeLeft.isOver}
      <div
        class="absolute top-4 right-4 px-4 py-2.5 bg-amber-500/90 text-white text-xs font-black uppercase tracking-widest rounded-full shadow-lg shadow-amber-500/40 border border-white/30 animate-glow-pulse flex items-center gap-2"
      >
        <Trophy class="w-3.5 h-3.5" />
        {$_("milestone.launched")}
      </div>
    {/if}

    <!-- Elapsed indicator -->
    {#if timeLeft.isOver}
      <div
        class="absolute bottom-4 right-4 text-[10px] font-bold text-white/40 uppercase tracking-widest"
      >
        ↑ {$_("milestone.elapsed")}
      </div>
    {/if}
  </div>
</div>

<ConfirmModal
  show={showHideConfirm}
  title={$_("milestone.hide_title")}
  message={$_("milestone.hide_confirm")}
  confirmText={$_("common.hide")}
  type="warning"
  onClose={() => (showHideConfirm = false)}
  onConfirm={handleToggleHide}
/>
