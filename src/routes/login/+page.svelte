<script lang="ts">
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { onDestroy, onMount } from "svelte";
  import {
    Mail,
    Lock,
    ArrowRight,
    CheckCircle2,
    X,
    ShieldCheck,
    ClipboardList,
    ListChecks,
    Users,
    BarChart3,
    Eye,
    EyeOff,
  } from "lucide-svelte";
  import favicon from "$lib/assets/favicon.svg";
  import LanguagePicker from "$lib/components/LanguagePicker.svelte";

  import { user } from "$lib/stores/auth";
  import { api } from "$lib/apis";
  import { _ } from "$lib/i18n";

  const HERO_TITLE_SEPARATOR = "\u0000";
  const HERO_LINE_1_TYPE_DELAY_MS = 34;
  const HERO_LINE_2_TYPE_DELAY_MS = 42;
  const HERO_LINE_2_DELETE_DELAY_MS = 24;
  const HERO_LINE_2_START_DELAY_MS = 220;
  const HERO_LINE_2_RESTART_DELAY_MS = 260;
  const HERO_VARIANT_PAUSE_MS = 1450;
  const HERO_TYPING_START_DELAY_MS = 180;

  type PreviewCase = {
    name: string;
    detail: string;
    badge: string;
    tone: "blue" | "amber" | "cyan";
  };

  type LoginSummary = {
    activeTasks: number;
    testCases: number;
    teammates: number;
    latestRunName: string;
    latestRunSubtitle: string;
    sampleCases: PreviewCase[];
  };

  const FALLBACK_SUMMARY: LoginSummary = {
    activeTasks: 128,
    testCases: 42,
    teammates: 8,
    latestRunName: "Test run regression",
    latestRunSubtitle: "34 passed, 3 blocked",
    sampleCases: [
      {
        name: "Release checklist",
        detail: "API, QA, deploy",
        badge: "Actual",
        tone: "blue",
      },
      {
        name: "Test run regression",
        detail: "34 passed, 3 blocked",
        badge: "Medium",
        tone: "amber",
      },
      {
        name: "Warehouse sync",
        detail: "offline queue ready",
        badge: "Ready",
        tone: "cyan",
      },
    ],
  };

  let email = "";
  let password = "";
  let showPassword = false;
  let loading = false;
  let error = "";

  let showPrivacyModal = false;
  let showTermsModal = false;
  let typedHeroTitle1 = "";
  let typedHeroTitle2 = "";
  let heroLine1TypingComplete = false;
  let heroTitleSignature = "";
  let typingTimer: ReturnType<typeof setTimeout> | null = null;
  let loginSummary: LoginSummary = FALLBACK_SUMMARY;

  async function handleLogin() {
    loading = true;
    error = "";

    try {
      const res = await api.auth.login(email, password);

      const data = await res.json();

      if (res.ok) {
        // Set cookie on client side
        document.cookie = `_khun_ph_token=${data.token}; path=/; max-age=${60 * 60 * 24 * 7}; samesite=Lax`;

        const userEmail = data.email || email;
        const userId = data.id || "";
        const userUuid = data.user_id || "";
        const userRole = data.role || "user";
        const profile = data.profile;

        localStorage.setItem("user_email", userEmail);
        if (userId) localStorage.setItem("user_id", userId);
        if (userUuid) localStorage.setItem("user_uuid", userUuid);
        localStorage.setItem("user_role", userRole);
        if (profile)
          localStorage.setItem("user_profile", JSON.stringify(profile));

        user.set({
          id: userId,
          email: userEmail,
          user_id: userUuid,
          role: userRole,
          profile: profile,
        });
        goto(`${base}/dashboard`);
      } else {
        error = data.error || $_("login__error_default");
      }
    } catch (e) {
      error = $_("login__error_default");
    } finally {
      loading = false;
    }
  }

  $: valueProps = [
    $_("login__value_prop_1"),
    $_("login__value_prop_2"),
    $_("login__value_prop_3"),
  ];

  $: heroTitle1 = $_("login__hero_title_1");
  $: heroTitle2 = $_("login__hero_title_2");
  $: if (getHeroTitleSignature(heroTitle1, heroTitle2) !== heroTitleSignature) {
    heroTitleSignature = getHeroTitleSignature(heroTitle1, heroTitle2);
    startHeroTyping(heroTitle1, heroTitle2);
  }

  function getHeroTitleSignature(line1: string, line2: string) {
    return `${line1}${HERO_TITLE_SEPARATOR}${line2}`;
  }

  function getHeroTitleVariants(baseLine: string) {
    if (!baseLine.toLowerCase().includes("workspace")) return [baseLine];

    return [
      baseLine,
      baseLine.replace(/workspace/i, "tasks"),
      baseLine.replace(/workspace/i, "test cases"),
      baseLine.replace(/workspace/i, "team"),
    ];
  }

  function clearTypingTimer() {
    if (!typingTimer) return;

    clearTimeout(typingTimer);
    typingTimer = null;
  }

  function startHeroTyping(line1: string, line2: string) {
    clearTypingTimer();

    const shouldReduceMotion =
      browser &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (!browser || shouldReduceMotion) {
      typedHeroTitle1 = line1;
      typedHeroTitle2 = line2;
      heroLine1TypingComplete = true;
      return;
    }

    const line2Variants = getHeroTitleVariants(line2);
    typedHeroTitle1 = "";
    typedHeroTitle2 = "";
    heroLine1TypingComplete = false;

    const typeLine1 = (step = 0) => {
      typedHeroTitle1 = line1.slice(0, step);
      if (step < line1.length) {
        step += 1;
        typingTimer = setTimeout(() => typeLine1(step), HERO_LINE_1_TYPE_DELAY_MS);
        return;
      }

      heroLine1TypingComplete = true;
      typingTimer = setTimeout(() => typeLine2(0, 0), HERO_LINE_2_START_DELAY_MS);
    };

    const typeLine2 = (variantIndex: number, step = 0) => {
      const phrase = line2Variants[variantIndex];
      typedHeroTitle2 = phrase.slice(0, step);

      if (step < phrase.length) {
        typingTimer = setTimeout(
          () => typeLine2(variantIndex, step + 1),
          HERO_LINE_2_TYPE_DELAY_MS,
        );
        return;
      }

      typingTimer = setTimeout(
        () => deleteLine2(variantIndex, phrase.length),
        HERO_VARIANT_PAUSE_MS,
      );
    };

    const deleteLine2 = (variantIndex: number, step: number) => {
      const phrase = line2Variants[variantIndex];
      typedHeroTitle2 = phrase.slice(0, step);

      if (step > 0) {
        typingTimer = setTimeout(
          () => deleteLine2(variantIndex, step - 1),
          HERO_LINE_2_DELETE_DELAY_MS,
        );
        return;
      }

      const nextVariantIndex = (variantIndex + 1) % line2Variants.length;
      typingTimer = setTimeout(
        () => typeLine2(nextVariantIndex, 0),
        HERO_LINE_2_RESTART_DELAY_MS,
      );
    };

    typingTimer = setTimeout(() => typeLine1(0), HERO_TYPING_START_DELAY_MS);
  }

  function getLoginSummaryWorkspaceId() {
    if (!browser) return "";

    const params = new URLSearchParams(window.location.search);
    return (
      params.get("workspace_id") ||
      params.get("ws_id") ||
      localStorage.getItem("current-workspace-id") ||
      ""
    );
  }

  function formatRunSubtitle(stats: any) {
    if (!stats) return FALLBACK_SUMMARY.latestRunSubtitle;

    const parts = [
      ["passed", stats.passed],
      ["failed", stats.failed],
      ["blocked", stats.blocked],
      ["pending", stats.pending],
    ]
      .filter(([, count]) => Number(count) > 0)
      .slice(0, 2)
      .map(([label, count]) => `${count} ${label}`);

    return parts.length > 0 ? parts.join(", ") : `${stats.total || 0} cases`;
  }

  function getCaseTone(index: number): PreviewCase["tone"] {
    return (["blue", "amber", "cyan"] as const)[index % 3];
  }

  function formatPreviewCase(testCase: any, index: number): PreviewCase {
    const status = testCase?.status || "actual";
    const priority = testCase?.priority || "medium";
    const testNo = testCase?.test_no ? `TC-${testCase.test_no}` : "Test case";

    return {
      name: testCase?.name || FALLBACK_SUMMARY.sampleCases[index]?.name || "Test case",
      detail: `${testNo}, ${priority}`,
      badge: status.charAt(0).toUpperCase() + status.slice(1),
      tone: getCaseTone(index),
    };
  }

  async function loadPublicLoginSummary() {
    const workspaceId = getLoginSummaryWorkspaceId();
    if (!workspaceId || workspaceId === "__my_tasks__") return;

    try {
      const response = await api.public.getSummary(workspaceId);
      if (!response.ok) return;

      const data = await response.json();
      const sampleCases = (data.sample_cases || [])
        .slice(0, 3)
        .map(formatPreviewCase);

      loginSummary = {
        activeTasks: Number(data.active_tasks ?? FALLBACK_SUMMARY.activeTasks),
        testCases: Number(data.test_cases ?? FALLBACK_SUMMARY.testCases),
        teammates: Number(data.teammates ?? FALLBACK_SUMMARY.teammates),
        latestRunName:
          data.latest_run?.name || FALLBACK_SUMMARY.latestRunName,
        latestRunSubtitle: data.latest_run?.stats
          ? formatRunSubtitle(data.latest_run.stats)
          : FALLBACK_SUMMARY.latestRunSubtitle,
        sampleCases:
          sampleCases.length > 0 ? sampleCases : FALLBACK_SUMMARY.sampleCases,
      };
    } catch (e) {
      loginSummary = FALLBACK_SUMMARY;
    }
  }

  function glassTilt(node: HTMLElement) {
    const shouldReduceMotion =
      browser &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (shouldReduceMotion) return {};

    const maxTilt = Number(node.dataset.tiltMax || 10);
    let frame = 0;

    const setIdle = () => {
      node.style.setProperty("--tilt-x", "0deg");
      node.style.setProperty("--tilt-y", "0deg");
      node.style.setProperty("--lift", "0px");
      node.style.setProperty("--depth", "0px");
      node.style.setProperty("--sheen-x", "50%");
      node.style.setProperty("--sheen-y", "50%");
    };

    const handlePointerMove = (event: PointerEvent) => {
      if (frame) cancelAnimationFrame(frame);

      frame = requestAnimationFrame(() => {
        const rect = node.getBoundingClientRect();
        const x = (event.clientX - rect.left) / rect.width;
        const y = (event.clientY - rect.top) / rect.height;
        const rotateY = (x - 0.5) * maxTilt * 2;
        const rotateX = (0.5 - y) * maxTilt * 2;

        node.style.setProperty("--tilt-x", `${rotateX.toFixed(2)}deg`);
        node.style.setProperty("--tilt-y", `${rotateY.toFixed(2)}deg`);
        node.style.setProperty("--lift", "-7px");
        node.style.setProperty("--depth", "28px");
        node.style.setProperty("--sheen-x", `${(x * 100).toFixed(1)}%`);
        node.style.setProperty("--sheen-y", `${(y * 100).toFixed(1)}%`);
      });
    };

    const handlePointerLeave = () => {
      if (frame) cancelAnimationFrame(frame);
      setIdle();
    };

    setIdle();
    node.addEventListener("pointermove", handlePointerMove);
    node.addEventListener("pointerleave", handlePointerLeave);
    node.addEventListener("blur", handlePointerLeave);

    return {
      destroy() {
        if (frame) cancelAnimationFrame(frame);
        node.removeEventListener("pointermove", handlePointerMove);
        node.removeEventListener("pointerleave", handlePointerLeave);
        node.removeEventListener("blur", handlePointerLeave);
      },
    };
  }

  onDestroy(clearTypingTimer);

  onMount(() => {
    void loadPublicLoginSummary();
  });
</script>

<svelte:head>
  <title>{$_("meta__login_title")}</title>
  <meta name="description" content={$_("meta__login_desc")} />
</svelte:head>

<div class="login-shell relative isolate min-h-screen w-full overflow-hidden bg-[#050816] text-white">
  <div class="login-grid absolute inset-0 -z-10"></div>
  <div class="login-horizon absolute inset-x-0 top-0 -z-10 h-px"></div>
  <div
    class="mx-auto flex min-h-screen w-full max-w-7xl flex-col px-4 py-4 sm:px-6 lg:px-8"
  >
    <header
      class="flex items-center justify-between rounded-xl border border-white/10 bg-slate-950/35 px-3 py-3 shadow-2xl shadow-black/20 backdrop-blur-xl sm:px-4"
    >
      <div class="flex min-w-0 items-center gap-3">
        <div
          class="grid h-10 w-10 shrink-0 place-items-center rounded-lg border border-blue-300/25 bg-blue-500/10 shadow-lg shadow-blue-950/30"
        >
          <img
            src={favicon}
            alt="Khun Phaen Logo"
            class="h-6 w-6 object-contain"
          />
        </div>
        <div class="min-w-0">
          <p class="truncate text-base font-black tracking-tight text-white">
            {$_("login__brand_name")}
          </p>
          <p class="hidden text-xs font-semibold text-slate-400 sm:block">
            Offline-first task operations
          </p>
        </div>
      </div>

      <LanguagePicker isDark={true} />
    </header>

    <main
      class="grid flex-1 items-center gap-8 py-10 lg:grid-cols-[minmax(0,1.08fr)_minmax(390px,0.72fr)] lg:gap-14"
    >
      <section class="hidden min-w-0 lg:block">
        <div class="max-w-2xl">
          <div
            class="mb-7 inline-flex items-center gap-2 rounded-full border border-blue-300/20 bg-blue-500/10 px-3.5 py-2 text-xs font-black text-blue-100/80 shadow-lg shadow-blue-950/20"
          >
            <ShieldCheck size={14} class="text-blue-300" />
            Secure workspace access
          </div>

          <h1
            class="max-w-2xl text-6xl font-black leading-[1.02] text-white"
            aria-label={`${heroTitle1} ${heroTitle2}`}
          >
            <span aria-hidden="true">
              {typedHeroTitle1}
              {#if !heroLine1TypingComplete}
                <span class="typing-caret"></span>
              {/if}
            </span>
            <span class="block text-blue-200" aria-hidden="true">
              {typedHeroTitle2}
              {#if heroLine1TypingComplete}
                <span class="typing-caret"></span>
              {/if}
            </span>
          </h1>
          <p class="mt-6 max-w-xl text-base leading-8 text-slate-300">
            {$_("login__hero_desc")}
          </p>

          <div class="mt-10 grid max-w-2xl grid-cols-3 gap-3">
            <div
              class="premium-tile glass-tilt rounded-lg border border-white/10 bg-slate-900/45 p-4"
              data-tilt-max="9"
              use:glassTilt
            >
              <ClipboardList class="mb-3 text-blue-300" size={20} />
              <p class="text-2xl font-black text-white">{loginSummary.activeTasks}</p>
              <p class="mt-1 text-xs font-semibold text-slate-400">
                active tasks
              </p>
            </div>
            <div
              class="premium-tile glass-tilt rounded-lg border border-white/10 bg-slate-900/45 p-4"
              data-tilt-max="9"
              use:glassTilt
            >
              <ListChecks class="mb-3 text-indigo-300" size={20} />
              <p class="text-2xl font-black text-white">{loginSummary.testCases}</p>
              <p class="mt-1 text-xs font-semibold text-slate-400">
                test cases
              </p>
            </div>
            <div
              class="premium-tile glass-tilt rounded-lg border border-white/10 bg-slate-900/45 p-4"
              data-tilt-max="9"
              use:glassTilt
            >
              <Users class="mb-3 text-cyan-300" size={20} />
              <p class="text-2xl font-black text-white">{loginSummary.teammates}</p>
              <p class="mt-1 text-xs font-semibold text-slate-400">
                teammates
              </p>
            </div>
          </div>
        </div>

        <div
          class="premium-panel glass-tilt mt-12 overflow-hidden rounded-xl border border-white/10 bg-[#0b1020]/95 shadow-2xl shadow-black/40"
          data-tilt-max="5"
          use:glassTilt
        >
          <div
            class="flex items-center justify-between border-b border-white/10 bg-white/[0.025] px-4 py-3"
          >
            <div class="flex items-center gap-2">
              <span class="h-2.5 w-2.5 rounded-full bg-rose-400"></span>
              <span class="h-2.5 w-2.5 rounded-full bg-amber-400"></span>
              <span class="h-2.5 w-2.5 rounded-full bg-emerald-400"></span>
            </div>
            <div
              class="flex max-w-[260px] items-center gap-2 rounded-full border border-white/10 px-3 py-1 text-xs font-bold text-slate-400"
              title={loginSummary.latestRunSubtitle}
            >
              <BarChart3 size={13} />
              <span class="truncate">{loginSummary.latestRunName}</span>
            </div>
          </div>

          <div class="grid gap-4 p-4 xl:grid-cols-[0.75fr_1.25fr]">
            <div class="space-y-3">
              {#each valueProps as prop}
                <div
                  class="flex items-start gap-3 rounded-lg border border-white/10 bg-slate-950/25 p-3"
                >
                  <CheckCircle2 class="mt-0.5 shrink-0 text-blue-300" size={17} />
                  <span class="text-sm font-semibold leading-5 text-slate-300">
                    {prop}
                  </span>
                </div>
              {/each}
            </div>

            <div class="grid gap-3">
              {#each loginSummary.sampleCases as item (item.name)}
                <div
                  class="grid grid-cols-[1fr_auto] gap-3 rounded-lg border border-white/10 bg-slate-950/25 p-3"
                >
                  <div class="min-w-0">
                    <p class="truncate text-sm font-black text-white">
                      {item.name}
                    </p>
                    <p class="mt-1 text-xs font-semibold text-slate-500">
                      {item.detail}
                    </p>
                  </div>
                  <span
                    class="h-fit rounded-full px-2 py-1 text-xs font-black {item.tone ===
                    'amber'
                      ? 'bg-amber-400/10 text-amber-200'
                      : item.tone === 'cyan'
                        ? 'bg-cyan-400/10 text-cyan-200'
                        : 'bg-blue-500/10 text-blue-200'}"
                    >{item.badge}</span
                  >
                </div>
              {/each}
            </div>
          </div>
        </div>
      </section>

      <section class="mx-auto w-full max-w-md">
        <div
          class="premium-panel rounded-xl border border-white/10 bg-[#0c1222]/95 p-5 shadow-2xl shadow-black/45 sm:p-7"
        >
          <div class="mb-7">
            <div
              class="mb-5 inline-flex h-12 w-12 items-center justify-center rounded-lg border border-blue-400/20 bg-blue-500/10 lg:hidden"
            >
              <img
                src={favicon}
                alt="Khun Phaen Logo"
                class="h-7 w-7 object-contain"
              />
            </div>
            <p class="text-sm font-black text-blue-200">{$_("login__subtitle")}</p>
            <h2 class="mt-2 text-4xl font-black tracking-tight text-white">
              {$_("login__title")}
            </h2>
            <div class="mt-5 grid grid-cols-3 gap-2 lg:hidden">
              <div
                class="rounded-lg border border-white/10 bg-slate-950/35 p-3"
              >
                <ClipboardList class="mb-2 text-blue-300" size={17} />
                <p class="text-xl font-black text-white">
                  {loginSummary.activeTasks}
                </p>
                <p class="mt-1 truncate text-[10px] font-black uppercase tracking-wide text-slate-500">
                  tasks
                </p>
              </div>
              <div
                class="rounded-lg border border-white/10 bg-slate-950/35 p-3"
              >
                <ListChecks class="mb-2 text-indigo-300" size={17} />
                <p class="text-xl font-black text-white">
                  {loginSummary.testCases}
                </p>
                <p class="mt-1 truncate text-[10px] font-black uppercase tracking-wide text-slate-500">
                  cases
                </p>
              </div>
              <div
                class="rounded-lg border border-white/10 bg-slate-950/35 p-3"
              >
                <Users class="mb-2 text-cyan-300" size={17} />
                <p class="text-xl font-black text-white">
                  {loginSummary.teammates}
                </p>
                <p class="mt-1 truncate text-[10px] font-black uppercase tracking-wide text-slate-500">
                  team
                </p>
              </div>
            </div>
          </div>

          <form on:submit|preventDefault={handleLogin} class="space-y-5">
            <div>
              <label
                for="email"
                class="mb-2 block text-sm font-bold text-slate-200"
                >{$_("login__email_label")}</label
              >
              <div class="relative">
                <Mail
                  size={18}
                  class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-500"
                />
                <input
                  type="email"
                  id="email"
                  bind:value={email}
                  required
                  autocomplete="email"
                  placeholder={$_("login__email_placeholder")}
                  class="h-12 w-full rounded-lg border border-slate-700/80 bg-slate-950/55 pl-11 pr-4 text-sm font-semibold text-white outline-none transition-all placeholder:text-slate-500 hover:border-slate-600 focus:border-blue-400/70 focus:bg-slate-950 focus:ring-4 focus:ring-blue-500/15"
                />
              </div>
            </div>

            <div>
              <label
                for="password"
                class="mb-2 block text-sm font-bold text-slate-200"
                >{$_("login__password_label")}</label
              >
              <div class="relative">
                <Lock
                  size={18}
                  class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-500"
                />
                <input
                  type={showPassword ? "text" : "password"}
                  id="password"
                  bind:value={password}
                  required
                  autocomplete="current-password"
                  placeholder="••••••••"
                  class="h-12 w-full rounded-lg border border-slate-700/80 bg-slate-950/55 pl-11 pr-12 text-sm font-semibold text-white outline-none transition-all placeholder:text-slate-500 hover:border-slate-600 focus:border-blue-400/70 focus:bg-slate-950 focus:ring-4 focus:ring-blue-500/15"
                />
                <button
                  type="button"
                  class="absolute right-2 top-1/2 grid h-8 w-8 -translate-y-1/2 place-items-center rounded-md text-slate-500 transition-colors hover:bg-white/5 hover:text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                  aria-label={showPassword ? "Hide password" : "Show password"}
                  title={showPassword ? "Hide password" : "Show password"}
                  on:click={() => (showPassword = !showPassword)}
                >
                  {#if showPassword}
                    <EyeOff size={17} />
                  {:else}
                    <Eye size={17} />
                  {/if}
                </button>
              </div>
            </div>

            {#if error}
              <div
                class="flex items-start gap-2 rounded-lg border border-rose-400/20 bg-rose-500/10 px-3 py-3 text-sm font-semibold text-rose-200 animate-fade-in"
                aria-live="polite"
              >
                <X class="mt-0.5 shrink-0" size={16} />
                <span>{error}</span>
              </div>
            {/if}

            <button
              type="submit"
              disabled={loading}
              class="group flex h-12 w-full items-center justify-center gap-2 rounded-lg bg-blue-600 px-4 text-sm font-black text-white shadow-xl shadow-blue-950/40 transition-all hover:-translate-y-0.5 hover:bg-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-500/20 disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:translate-y-0"
            >
              {#if loading}
                <span
                  class="h-5 w-5 rounded-full border-2 border-white/30 border-t-white animate-spin"
                ></span>
              {:else}
                {$_("login__btn_submit")}
                <ArrowRight
                  size={18}
                  class="transition-transform group-hover:translate-x-0.5"
                />
              {/if}
            </button>
          </form>

          <div
            class="mt-6 rounded-lg border border-white/10 bg-slate-950/25 px-3 py-3 text-center text-sm font-semibold text-slate-400"
          >
            {$_("login__invitation_only")}
          </div>
        </div>

        <footer
          class="mt-5 flex flex-wrap items-center justify-center gap-x-4 gap-y-2 text-xs font-semibold text-slate-500"
        >
          <button
            on:click={() => (showPrivacyModal = true)}
            type="button"
            class="transition-colors hover:text-white"
            >{$_("login__link_privacy")}</button
          >
          <button
            on:click={() => (showTermsModal = true)}
            type="button"
            class="transition-colors hover:text-white"
            >{$_("login__link_terms")}</button
          >
          <a
            href="https://fakduai.com/"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 transition-colors hover:text-white"
          >
            <img
              src="https://fakduai.com/favicon.ico"
              alt="Fakduai Logo"
              class="h-4 w-4 rounded-sm bg-white/10 object-contain p-0.5"
            />
            {$_("login__powered_by")} Fakduai
          </a>
        </footer>
      </section>
    </main>
  </div>
</div>

<!-- Privacy Modal -->
{#if showPrivacyModal}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in"
    on:click={() => (showPrivacyModal = false)}
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="bg-slate-900 border border-white/10 p-6 rounded-2xl max-w-lg w-full shadow-2xl relative"
      on:click|stopPropagation
    >
      <button
        class="absolute top-4 right-4 text-slate-400 hover:text-white transition-colors"
        on:click={() => (showPrivacyModal = false)}
      >
        <X size={20} />
      </button>
      <h2 class="text-xl font-bold text-white mb-4">
        {$_("login__privacy_title")}
      </h2>
      <div class="text-slate-300 text-sm leading-relaxed space-y-3">
        <p>{$_("login__privacy_content")}</p>
      </div>
    </div>
  </div>
{/if}

<!-- Terms Modal -->
{#if showTermsModal}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in"
    on:click={() => (showTermsModal = false)}
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="bg-slate-900 border border-white/10 p-6 rounded-2xl max-w-lg w-full shadow-2xl relative"
      on:click|stopPropagation
    >
      <button
        class="absolute top-4 right-4 text-slate-400 hover:text-white transition-colors"
        on:click={() => (showTermsModal = false)}
      >
        <X size={20} />
      </button>
      <h2 class="text-xl font-bold text-white mb-4">
        {$_("login__terms_title")}
      </h2>
      <div class="text-slate-300 text-sm leading-relaxed space-y-3">
        <p>{$_("login__terms_content")}</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .login-shell {
    background:
      linear-gradient(180deg, rgba(15, 23, 42, 0.32), rgba(2, 6, 23, 0) 34%),
      #050816;
  }

  .login-grid {
    background-image:
      linear-gradient(rgba(148, 163, 184, 0.045) 1px, transparent 1px),
      linear-gradient(90deg, rgba(148, 163, 184, 0.045) 1px, transparent 1px);
    background-size: 40px 40px;
    mask-image: linear-gradient(to bottom, black 0%, black 62%, transparent 100%);
  }

  .login-horizon {
    background: linear-gradient(
      90deg,
      transparent,
      rgba(96, 165, 250, 0.48),
      transparent
    );
  }

  .premium-panel,
  .premium-tile {
    position: relative;
    backdrop-filter: blur(18px);
  }

  .premium-panel::before,
  .premium-tile::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    pointer-events: none;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }

  .glass-tilt {
    --tilt-x: 0deg;
    --tilt-y: 0deg;
    --lift: 0px;
    --depth: 0px;
    --sheen-x: 50%;
    --sheen-y: 50%;
    isolation: isolate;
    overflow: hidden;
    transform-style: preserve-3d;
    transform: perspective(1000px) rotateX(var(--tilt-x))
      rotateY(var(--tilt-y)) translate3d(0, var(--lift), var(--depth));
    transition:
      transform 420ms cubic-bezier(0.2, 0.8, 0.2, 1),
      border-color 420ms cubic-bezier(0.2, 0.8, 0.2, 1),
      box-shadow 420ms cubic-bezier(0.2, 0.8, 0.2, 1),
      background-color 420ms cubic-bezier(0.2, 0.8, 0.2, 1);
    will-change: transform;
  }

  .glass-tilt::after {
    content: "";
    position: absolute;
    inset: -45%;
    z-index: -1;
    pointer-events: none;
    background:
      radial-gradient(circle at var(--sheen-x) var(--sheen-y), rgba(147, 197, 253, 0.22), transparent 26%),
      linear-gradient(115deg, transparent 34%, rgba(255, 255, 255, 0.16) 48%, transparent 62%);
    opacity: 0;
    transform: translate3d(0, 0, 42px) rotate(10deg);
    transition:
      opacity 420ms cubic-bezier(0.2, 0.8, 0.2, 1),
      transform 520ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .premium-tile {
    transform-origin: center;
  }

  @media (hover: hover) and (pointer: fine) {
    .glass-tilt:hover {
      border-color: rgba(147, 197, 253, 0.32);
      background-color: rgba(15, 23, 42, 0.72);
      box-shadow:
        0 30px 80px rgba(0, 0, 0, 0.44),
        0 0 0 1px rgba(147, 197, 253, 0.08),
        inset 0 1px 0 rgba(255, 255, 255, 0.12);
    }

    .glass-tilt:hover::after {
      opacity: 1;
    }

    .premium-panel.glass-tilt:hover {
      box-shadow:
        0 36px 96px rgba(0, 0, 0, 0.46),
        0 0 0 1px rgba(147, 197, 253, 0.08),
        inset 0 1px 0 rgba(255, 255, 255, 0.12);
    }
  }

  .typing-caret {
    display: inline-block;
    width: 0.08em;
    height: 0.82em;
    margin-left: 0.08em;
    transform: translateY(0.08em);
    border-radius: 999px;
    background: currentColor;
    animation: typing-caret-blink 900ms steps(1) infinite;
  }

  @keyframes typing-caret-blink {
    0%,
    48% {
      opacity: 1;
    }
    49%,
    100% {
      opacity: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .premium-tile,
    .glass-tilt,
    .glass-tilt::after,
    button,
    input,
    .typing-caret {
      transition: none !important;
      animation: none !important;
    }

    .premium-tile:hover,
    .glass-tilt:hover,
    button:hover {
      transform: none !important;
    }

    .glass-tilt:hover::after {
      opacity: 0 !important;
    }
  }

  /* CSS Fix for WebKit Autofill replacing backgrounds */
  input:-webkit-autofill,
  input:-webkit-autofill:hover,
  input:-webkit-autofill:focus,
  input:-webkit-autofill:active {
    -webkit-box-shadow: 0 0 0 30px #0f172a inset !important;
    -webkit-text-fill-color: white !important;
    caret-color: white !important;
    transition: background-color 5000s ease-in-out 0s;
  }
</style>
