<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { _ } from "svelte-i18n";
  import { locale } from "$lib/i18n";
  import { theme } from "$lib/stores/theme";
  import {
    Globe,
    KeyRound,
    Palette,
    Settings,
    ShieldCheck,
  } from "lucide-svelte";

  onMount(() => {
    if (!browser) return;
  });

  $: activeLocale = ($locale || "en").toUpperCase();

  const summaryCards = [
    {
      label: "settings__card_role_label",
      value: "settings__card_role_value",
      helper: "settings__card_role_helper",
      icon: ShieldCheck,
      tone:
        "from-emerald-500/15 via-emerald-500/5 to-transparent text-emerald-600 dark:text-emerald-300",
    },
    {
      label: "settings__card_locale_label",
      value: "settings__card_locale_value",
      helper: "settings__card_locale_helper",
      icon: Globe,
      tone:
        "from-sky-500/15 via-sky-500/5 to-transparent text-sky-600 dark:text-sky-300",
    },
    {
      label: "settings__card_theme_label",
      value: "settings__card_theme_value",
      helper: "settings__card_theme_helper",
      icon: Palette,
      tone:
        "from-fuchsia-500/15 via-fuchsia-500/5 to-transparent text-fuchsia-600 dark:text-fuchsia-300",
    },
    {
      label: "settings__card_security_label",
      value: "settings__card_security_value",
      helper: "settings__card_security_helper",
      icon: KeyRound,
      tone:
        "from-amber-500/15 via-amber-500/5 to-transparent text-amber-600 dark:text-amber-300",
    },
  ];

</script>

<div class="space-y-6 animate-fade-in">
  <section class="overflow-hidden rounded-[30px] border border-slate-800/80 bg-slate-950 text-white shadow-[0_30px_80px_rgba(15,23,42,0.45)]">
    <div class="bg-[radial-gradient(circle_at_top_left,_rgba(99,102,241,0.28),_transparent_35%),linear-gradient(135deg,_#020617_0%,_#0f172a_48%,_#1e1b4b_100%)] px-6 py-7 sm:px-8">
      <div class="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div class="min-w-0">
          <div class="mb-4 inline-flex rounded-2xl bg-white/10 p-3 ring-1 ring-white/15 backdrop-blur">
            <Settings size={22} />
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <h2 class="text-2xl font-bold tracking-tight sm:text-3xl">
              {$_("settings__general_title")}
            </h2>
            <span class="rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-200">
              {$_("settings__admin_only")}
            </span>
          </div>
          <p class="mt-3 max-w-2xl text-sm leading-6 text-slate-300/85">
            {$_("settings__general_subtitle")}
          </p>
        </div>

        <div class="grid w-full max-w-md grid-cols-2 gap-3">
          {#each summaryCards as card}
            <div class="rounded-2xl border border-white/10 bg-white/5 p-4 backdrop-blur">
              <div class={`inline-flex rounded-xl bg-gradient-to-br p-2.5 ${card.tone}`}>
                <svelte:component this={card.icon} size={16} />
              </div>
              <p class="mt-3 text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
                {$_(card.label)}
              </p>
              <p class="mt-1 text-base font-bold text-white">
                {card.label === "settings__card_locale_label"
                  ? activeLocale
                  : card.label === "settings__card_theme_label"
                    ? $theme
                    : $_(card.value)}
              </p>
              <p class="mt-1 text-xs text-slate-400">
                {$_(card.helper)}
              </p>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </section>

</div>

<style>
  .animate-fade-in {
    animation: fadeIn 0.35s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
