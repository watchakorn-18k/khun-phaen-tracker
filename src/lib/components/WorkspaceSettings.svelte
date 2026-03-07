<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { api } from "$lib/apis";
  import {
    currentWorkspaceName,
    currentWorkspaceColor,
    currentWorkspaceIcon,
    setWorkspaceId,
  } from "$lib/stores/workspace";
  import {
    X,
    Save,
    LayoutTemplate,
    Briefcase,
    Code2,
    Rocket,
    Zap,
    Heart,
    Target,
    Globe,
    Book,
    Camera,
    Coffee,
    Music,
    Smile,
    Bell,
    CheckCircle2,
    RefreshCcw,
    Settings,
    ClipboardList,
    Loader,
    FlaskConical,
    CircleCheckBig,
  } from "lucide-svelte";
  import { fade, scale, slide } from "svelte/transition";

  export let workspaceId: string;

  const dispatch = createEventDispatcher();

  let name = $currentWorkspaceName;
  let shortName = "";
  let color = $currentWorkspaceColor || "#6366f1";
  let icon = $currentWorkspaceIcon || "LayoutTemplate";

  // Notification Config
  let webhookUrl = "";
  let lineNotifyToken = "";
  let notifyOnCreate = false;
  let notifyOnStatusChange: string[] = [];
  let autoEnabled = false;
  let autoDays: number[] = [1, 2, 3, 4, 5];
  let autoTime = "09:00";

  // States
  let isLoading = true;
  let isSaving = false;

  const COLORS = [
    { name: "Slate", value: "#64748b" },
    { name: "Red", value: "#ef4444" },
    { name: "Orange", value: "#f97316" },
    { name: "Amber", value: "#f59e0b" },
    { name: "Emerald", value: "#10b981" },
    { name: "Teal", value: "#14b8a6" },
    { name: "Blue", value: "#3b82f6" },
    { name: "Indigo", value: "#6366f1" },
    { name: "Violet", value: "#8b5cf6" },
    { name: "Pink", value: "#ec4899" },
    { name: "Rose", value: "#f43f5e" },
  ];

  const ICONS = [
    { key: "LayoutTemplate", component: LayoutTemplate },
    { key: "Briefcase", component: Briefcase },
    { key: "Code2", component: Code2 },
    { key: "Rocket", component: Rocket },
    { key: "Zap", component: Zap },
    { key: "Heart", component: Heart },
    { key: "Target", component: Target },
    { key: "Globe", component: Globe },
    { key: "Book", component: Book },
    { key: "Camera", component: Camera },
    { key: "Coffee", component: Coffee },
    { key: "Music", component: Music },
    { key: "Smile", component: Smile },
  ];

  onMount(async () => {
    shortName = generateShortName(name);
    try {
      const res = await api.workspaces.getNotificationConfig(workspaceId);
      if (res.ok) {
        const data = await res.json();
        if (data.success && data.config) {
          webhookUrl = data.config.discord_webhook_url || "";
          lineNotifyToken = data.config.line_notify_token || "";
          notifyOnCreate = data.config.notify_on_create || false;
          notifyOnStatusChange = data.config.notify_on_status_change || [];
          autoEnabled = data.config.enabled || false;
          autoDays = data.config.days || [1, 2, 3, 4, 5];
          autoTime = data.config.time || "09:00";
        }
      }
    } catch (err) {
      console.error("Failed to load workspace settings:", err);
    } finally {
      isLoading = false;
    }
  });

  function normalizeShortName(value: string): string {
    return value.replace(/\s+/g, "").slice(0, 4).toUpperCase();
  }

  function generateShortName(value: string): string {
    return normalizeShortName(value.trim());
  }

  function resolveShortName(): string {
    const normalized = normalizeShortName(shortName);
    if (normalized) return normalized;
    return generateShortName(name);
  }

  async function handleSave() {
    isSaving = true;
    try {
      // 1. Update Basic Info
      const updateRes = await api.workspaces.update(
        workspaceId,
        name,
        color,
        icon,
        resolveShortName(),
      );
      if (updateRes.ok) {
        setWorkspaceId(
          workspaceId,
          name,
          undefined,
          color,
          icon,
          resolveShortName(),
        );
      }

      // 2. Update Notification Config
      await api.workspaces.updateNotificationConfig(workspaceId, {
        discord_webhook_url: webhookUrl,
        line_notify_token: lineNotifyToken,
        notify_on_create: notifyOnCreate,
        notify_on_status_change: notifyOnStatusChange,
        enabled: autoEnabled,
        days: autoDays,
        time: autoTime,
      });

      dispatch("workspaceUpdated");
      dispatch("close");
    } catch (err) {
      console.error("Failed to save workspace settings:", err);
    } finally {
      isSaving = false;
    }
  }

  function toggleStatus(status: string) {
    if (notifyOnStatusChange.includes(status)) {
      notifyOnStatusChange = notifyOnStatusChange.filter((s) => s !== status);
    } else {
      notifyOnStatusChange = [...notifyOnStatusChange, status];
    }
  }
</script>

<div
  class="fixed inset-0 z-[1000] flex items-center justify-center bg-slate-950/60 p-4 backdrop-blur-md"
  transition:fade={{ duration: 200 }}
>
  <div
    class="relative flex max-h-[90vh] w-full max-w-3xl flex-col overflow-hidden rounded-[32px] border border-slate-200 bg-white shadow-[0_28px_80px_rgba(15,23,42,0.32)] dark:border-gray-700 dark:bg-[#081224]"
    transition:scale={{ duration: 300, start: 0.95 }}
  >
    <!-- Header -->
    <div
      class="flex items-start justify-between gap-4 border-b border-slate-200 bg-[radial-gradient(circle_at_top_left,_rgba(59,130,246,0.22),_transparent_38%),linear-gradient(135deg,_#f8fafc_0%,_#e2e8f0_45%,_#c7d2fe_100%)] px-6 py-5 text-slate-900 dark:border-gray-700 dark:bg-gradient-to-br dark:from-slate-900 dark:via-slate-800 dark:to-indigo-950 dark:text-white sm:px-8 sm:py-6"
    >
      <div class="flex items-center gap-4">
        <div class="rounded-2xl bg-white/70 p-3 text-primary ring-1 ring-slate-200/80 backdrop-blur dark:bg-white/10 dark:text-white dark:ring-white/15">
          <Settings size={24} />
        </div>
        <div>
          <h2
            class="text-2xl font-black tracking-tight text-slate-900 dark:text-white"
          >
            {$_("workspaceSettings__title")}
          </h2>
          <p class="mt-1 text-sm font-medium text-slate-600 dark:text-slate-200/80">
            {$_("workspaceSettings__subtitle")}
          </p>
        </div>
      </div>
      <button
        on:click={() => dispatch("close")}
        class="rounded-2xl p-2 text-slate-500 transition-colors hover:bg-white/70 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-white/10 dark:hover:text-white"
      >
        <X size={24} />
      </button>
    </div>

    {#if isLoading}
      <div class="flex flex-1 flex-col items-center justify-center gap-4 py-20">
        <RefreshCcw size={40} class="animate-spin text-primary opacity-50" />
        <p class="animate-pulse font-medium text-slate-500 dark:text-slate-300">
          {$_("page__checking_access")}
        </p>
      </div>
    {:else}
      <div class="flex-1 overflow-y-auto bg-slate-50/70 p-5 dark:bg-transparent sm:p-8">
        <!-- Appearance Section -->
        <section class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-[0_12px_36px_rgba(15,23,42,0.06)] dark:border-gray-700/80 dark:bg-slate-950/40 dark:shadow-none">
          <div
            class="flex items-center gap-2 text-xs font-black uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500"
          >
            <span class="h-px w-8 bg-slate-200 dark:bg-gray-700"></span>
            {$_("workspaceSettings__appearance_title")}
          </div>

          <div class="mt-6 grid grid-cols-1 gap-8 md:grid-cols-2">
            <div class="space-y-4">
              <label
                for="ws-name"
                class="block text-sm font-bold text-slate-700 dark:text-gray-300"
                >{$_("workspaceSettings__name_label")}</label
              >
              <input
                id="ws-name"
                type="text"
                bind:value={name}
                class="w-full rounded-2xl border border-slate-200 bg-slate-100/80 px-4 py-3 font-bold text-slate-900 transition-all focus:border-primary/40 focus:bg-white focus:ring-2 focus:ring-primary/20 dark:border-gray-700 dark:bg-gray-800/90 dark:text-white dark:focus:bg-gray-800"
                placeholder={$_("workspaceSettings__name_placeholder")}
              />
            </div>

            <div class="space-y-4">
              <label
                for="ws-short-name"
                class="block text-sm font-bold text-slate-700 dark:text-gray-300"
                >ชื่อย่อ (สูงสุด 4 ตัว)</label
              >
              <input
                id="ws-short-name"
                type="text"
                value={shortName}
                maxlength="4"
                on:input={(e) =>
                  (shortName = normalizeShortName(
                    (e.currentTarget as HTMLInputElement).value,
                  ))}
                class="w-full rounded-2xl border border-slate-200 bg-slate-100/80 px-4 py-3 font-black uppercase tracking-widest text-slate-900 transition-all focus:border-primary/40 focus:bg-white focus:ring-2 focus:ring-primary/20 dark:border-gray-700 dark:bg-gray-800/90 dark:text-white dark:focus:bg-gray-800"
                placeholder={generateShortName(name) || "WORK"}
              />
            </div>

            <div class="space-y-4">
              <span
                class="block text-sm font-bold text-slate-700 dark:text-gray-300"
                >{$_("workspaceSettings__color_label")}</span
              >
              <div class="flex flex-wrap gap-2.5">
                {#each COLORS as c}
                  <button
                    on:click={() => (color = c.value)}
                    class="h-9 w-9 rounded-full border-2 transition-all hover:scale-110 {color ===
                    c.value
                      ? 'border-primary ring-4 ring-primary/15 scale-110'
                      : 'border-white shadow-sm dark:border-transparent'}"
                    style="background-color: {c.value}"
                    title={c.name}
                  ></button>
                {/each}
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <span
              class="block text-sm font-bold text-slate-700 dark:text-gray-300"
              >{$_("workspaceSettings__icon_label")}</span
            >
            <div class="grid grid-cols-4 gap-3 sm:grid-cols-6 md:grid-cols-7">
              {#each ICONS as ico}
                <button
                  on:click={() => (icon = ico.key)}
                  class="flex items-center justify-center rounded-2xl border p-3 transition-all hover:bg-slate-50 dark:hover:bg-gray-800 {icon ===
                  ico.key
                    ? 'border-primary bg-primary/10 text-primary shadow-[0_10px_24px_rgba(79,70,229,0.15)] dark:border-primary dark:bg-primary/10'
                    : 'border-slate-200 text-slate-400 dark:border-gray-700 dark:text-gray-400'}"
                >
                  <svelte:component this={ico.component} size={24} />
                </button>
              {/each}
            </div>
          </div>
        </section>

        <!-- Notification Section -->
        <section class="mt-8 rounded-[28px] border border-slate-200 bg-white p-6 shadow-[0_12px_36px_rgba(15,23,42,0.06)] dark:border-gray-700/80 dark:bg-slate-950/40 dark:shadow-none">
          <div
            class="flex items-center gap-2 text-xs font-black uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500"
          >
            <span class="h-px w-8 bg-slate-200 dark:bg-gray-700"></span>
            {$_("workspaceSettings__channels_title")}
          </div>

          <div class="mt-6 space-y-4">
            <div class="flex flex-col gap-2">
              <label
                for="discord-url"
                class="flex items-center gap-2 text-sm font-bold text-slate-700 dark:text-gray-300"
              >
                <Globe size={16} class="text-[#5865F2]" />
                {$_("workspaceSettings__discord_webhook_label")}
              </label>
              <input
                id="discord-url"
                type="text"
                bind:value={webhookUrl}
                class="w-full rounded-2xl border border-slate-200 bg-slate-100/80 px-4 py-3 text-sm font-medium text-slate-900 transition-all focus:border-[#5865F2]/40 focus:bg-white focus:ring-2 focus:ring-[#5865F2]/20 dark:border-gray-700 dark:bg-gray-800/90 dark:text-white dark:focus:bg-gray-800"
                placeholder={$_(
                  "workspaceSettings__discord_webhook_placeholder",
                )}
              />
            </div>

            <div class="flex flex-col gap-2">
              <label
                for="line-token"
                class="flex items-center gap-2 text-sm font-bold text-slate-700 dark:text-gray-300"
              >
                <Bell size={16} class="text-[#06C755]" />
                {$_("workspaceSettings__line_notify_token_label")}
              </label>
              <input
                id="line-token"
                type="password"
                bind:value={lineNotifyToken}
                class="w-full rounded-2xl border border-slate-200 bg-slate-100/80 px-4 py-3 text-sm font-medium text-slate-900 transition-all focus:border-[#06C755]/40 focus:bg-white focus:ring-2 focus:ring-[#06C755]/20 dark:border-gray-700 dark:bg-gray-800/90 dark:text-white dark:focus:bg-gray-800"
                placeholder={$_(
                  "workspaceSettings__line_notify_token_placeholder",
                )}
              />
            </div>
          </div>

          <div class="rounded-3xl border border-slate-200 bg-slate-50/80 p-6 dark:border-gray-700 dark:bg-gray-800/35">
            <div class="flex items-center justify-between">
              <div>
                <h4
                  class="flex items-center gap-2 text-sm font-black text-slate-900 dark:text-white"
                >
                  <RefreshCcw size={16} class="text-amber-500" />
                  {$_("workspaceSettings__auto_title")}
                </h4>
                <p
                  class="mt-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400"
                >
                  {$_("workspaceSettings__auto_subtitle")}
                </p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={autoEnabled}
                  class="sr-only peer"
                />
                <div
                  class="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-amber-500"
                ></div>
              </label>
            </div>

            {#if autoEnabled}
              <div
                class="grid grid-cols-1 md:grid-cols-2 gap-6"
                transition:slide
              >
                <div class="space-y-3">
                  <span
                    class="text-[10px] font-black uppercase tracking-widest text-slate-400 dark:text-slate-500"
                    >{$_("workspaceSettings__auto_days_label")}</span
                  >
                  <div class="flex flex-wrap gap-1.5">
                    {#each [1, 2, 3, 4, 5, 6, 0] as d}
                      <button
                        on:click={() => {
                          if (autoDays.includes(d))
                            autoDays = autoDays.filter((day) => day !== d);
                          else autoDays = [...autoDays, d].sort();
                        }}
                        class="px-2.5 py-1.5 rounded-lg text-[10px] font-black transition-all {autoDays.includes(
                          d,
                        )
                          ? 'bg-amber-500 text-white shadow-md shadow-amber-500/20'
                          : 'border border-slate-200 bg-white text-slate-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400'}"
                      >
                        {$_(
                          `calendarView__day_${["sun", "mon", "tue", "wed", "thu", "fri", "sat"][d]}`,
                        )}
                      </button>
                    {/each}
                  </div>
                </div>
                <div class="space-y-3">
                  <span
                    class="text-[10px] font-black uppercase tracking-widest text-gray-400"
                    >{$_("workspaceSettings__auto_time_label")}</span
                  >
                  <input
                    type="time"
                    bind:value={autoTime}
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-xl font-black text-slate-800 dark:border-gray-700 dark:bg-gray-800 dark:text-white"
                  />
                </div>
              </div>
            {/if}
          </div>

          <div class="mt-6 rounded-3xl border border-slate-200 bg-slate-50/80 p-6 dark:border-gray-700 dark:bg-gray-800/35">
            <div class="flex items-center justify-between">
              <div>
                <h4
                  class="flex items-center gap-2 text-sm font-black text-slate-900 dark:text-white"
                >
                  <CheckCircle2 size={16} class="text-primary" />
                  {$_("workspaceSettings__alerts_title")}
                </h4>
                <p
                  class="mt-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400"
                >
                  {$_("workspaceSettings__alerts_subtitle")}
                </p>
              </div>
            </div>

            <div class="space-y-4">
              <div
                class="flex items-center justify-between rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-full bg-emerald-100 dark:bg-emerald-500/20 text-emerald-600 flex items-center justify-center"
                  >
                    <Rocket size={16} />
                  </div>
                  <span
                    class="text-sm font-bold text-slate-700 dark:text-gray-200"
                    >{$_("workspaceSettings__alerts_create_label")}</span
                  >
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    bind:checked={notifyOnCreate}
                    class="sr-only peer"
                  />
                  <div
                    class="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary"
                  ></div>
                </label>
              </div>

              <div class="space-y-3">
                <span
                  class="ml-1 text-xs font-bold text-slate-500 dark:text-gray-400"
                  >{$_("workspaceSettings__alerts_status_label")}</span
                >
                <div class="flex flex-wrap gap-2">
                  {#each ["todo", "in-progress", "in-test", "done"] as status}
                    <button
                      on:click={() => toggleStatus(status)}
                      class="px-4 py-2 rounded-xl text-xs font-bold transition-all border-2 flex items-center gap-1.5 {notifyOnStatusChange.includes(
                        status,
                      )
                        ? 'bg-primary border-primary text-white shadow-lg shadow-primary/20'
                        : 'bg-white dark:bg-gray-800 border-slate-200 dark:border-gray-700 text-slate-500 dark:text-gray-400 hover:border-slate-300 dark:hover:border-gray-600'}"
                    >
                      {#if status === "todo"}
                        <ClipboardList size={14} />
                      {:else if status === "in-progress"}
                        <Loader size={14} />
                      {:else if status === "in-test"}
                        <FlaskConical size={14} />
                      {:else}
                        <CircleCheckBig size={14} />
                      {/if}
                      {$_(`page__filter_status_${status.replace("-", "_")}`)}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div
        class="flex items-center justify-end gap-3 border-t border-slate-200 bg-white/90 px-6 py-5 backdrop-blur dark:border-gray-700 dark:bg-slate-950/85 sm:px-8 sm:py-6"
      >
        <button
          on:click={() => dispatch("close")}
          class="px-6 py-3 text-sm font-bold text-slate-500 transition-colors hover:text-slate-900 dark:text-slate-300 dark:hover:text-white"
        >
          {$_("dashboard__modal_btn_cancel")}
        </button>
        <button
          on:click={handleSave}
          disabled={isSaving || !name.trim()}
          class="flex items-center gap-2 rounded-2xl bg-primary px-8 py-3 text-sm font-bold text-white shadow-[0_16px_40px_rgba(79,70,229,0.28)] transition-all hover:bg-primary-dark active:scale-95 disabled:opacity-50"
        >
          {#if isSaving}
            <RefreshCcw size={18} class="animate-spin" />
            {$_("taskForm__comments_saving")}
          {:else}
            <Save size={18} />
            {$_("workspaceSettings__btn_save")}
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>
