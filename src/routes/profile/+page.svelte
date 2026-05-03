<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { browser } from "$app/environment";
  import { _ } from "svelte-i18n";
  import {
    User,
    Lock,
    Mail,
    Save,
    Fingerprint,
    AtSign,
    UserCircle2,
    ShieldCheck,
    Monitor,
    Server,
    Database,
    Smartphone,
    Palette,
    CheckCircle2,
    Layers,
    Terminal,
  } from "lucide-svelte";
  import { api } from "$lib/apis";
  import { user, authLoading } from "$lib/stores/auth";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";

  let loading = false;
  let saveSuccess = false;
  let error = "";

  let firstName = "";
  let lastName = "";
  let nickname = "";
  let position = "";
  let discordId = "";
  let password = "";
  let confirmPassword = "";

  const positionOptions: Array<{ value: string; label: string; icon: any }> = [
    {
      value: "Full Stack Developer",
      label: "Full Stack Developer",
      icon: Layers,
    },
    { value: "Frontend Developer", label: "Frontend Developer", icon: Monitor },
    { value: "Backend Developer", label: "Backend Developer", icon: Server },
    { value: "DevOps", label: "DevOps", icon: Terminal },
    { value: "QA Tester", label: "QA Tester", icon: CheckCircle2 },
    { value: "Data Scientist", label: "Data Scientist", icon: Database },
    { value: "UX/UI Designer", label: "UX/UI Designer", icon: Palette },
    { value: "Mobile Developer", label: "Mobile Developer", icon: Smartphone },
  ];

  $: if (browser && !$authLoading && !$user) {
    goto(`${base}/login`);
  }

  onMount(() => {
    if ($user) {
      firstName = $user.profile?.first_name || "";
      lastName = $user.profile?.last_name || "";
      nickname = $user.profile?.nickname || "";
      position = $user.profile?.position || "";
      discordId = $user.discord_id || "";
    }
  });

  async function handleSubmit() {
    if (password && password !== confirmPassword) {
      error = $_("profileModal__password_mismatch");
      return;
    }

    loading = true;
    error = "";
    saveSuccess = false;

    try {
      const payload: Record<string, any> = {
        first_name: firstName,
        last_name: lastName,
        nickname,
        position,
        discord_id: discordId,
      };
      if (password) payload.password = password;

      const res = await api.auth.updateMe(payload);
      const data = await res.json();

      if (data.success) {
        saveSuccess = true;
        password = "";
        confirmPassword = "";
        user.update((u) => {
          if (!u) return u;
          return {
            ...u,
            discord_id: discordId,
            profile: {
              ...(u.profile || { profile_id: "", user_id: u.user_id }),
              first_name: firstName,
              last_name: lastName,
              nickname,
              position,
            },
          };
        });
        setTimeout(() => (saveSuccess = false), 3000);
      } else {
        error = data.error || $_("profileModal__save_error");
      }
    } catch (e) {
      console.error("Update profile failed:", e);
      error = $_("profileModal__save_error");
    } finally {
      loading = false;
    }
  }

  function getInitials(name: string) {
    return name
      .split(/\s+/)
      .map((w) => w[0]?.toUpperCase() ?? "")
      .slice(0, 2)
      .join("");
  }

  $: displayName =
    [$user?.profile?.first_name, $user?.profile?.last_name]
      .filter(Boolean)
      .join(" ") ||
    $user?.profile?.nickname ||
    $user?.email?.split("@")[0] ||
    "";
  $: initials = getInitials(displayName || $user?.email || "?");
</script>

<svelte:head>
  <title>{$_("meta__profile_title")}</title>
  <meta name="description" content={$_("meta__profile_desc")} />
</svelte:head>

<div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
  <Sidebar />

  <main
    class="flex-1 overflow-y-auto min-w-0 relative p-4 lg:p-8 custom-scrollbar"
  >
    <div class="max-w-7xl mx-auto">
      <div class="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
        <!-- Left Panel -->
        <aside class="lg:sticky lg:top-0 lg:self-start">
          <div
            class="overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-[0_18px_45px_rgba(15,23,42,0.08)] dark:border-gray-800 dark:bg-gray-800/50 dark:backdrop-blur-sm dark:shadow-none"
          >
            <!-- Gradient Header -->
            <div
              class="border-b border-slate-200 bg-[radial-gradient(circle_at_top_left,_rgba(99,102,241,0.22),_transparent_38%),linear-gradient(135deg,_#f8fafc_0%,_#e2e8f0_45%,_#c7d2fe_100%)] p-5 text-slate-900 dark:border-gray-700 dark:bg-gradient-to-br dark:from-slate-900 dark:via-slate-800 dark:to-indigo-950 dark:text-white"
            >
              <!-- Avatar -->
              <div class="mb-4 flex flex-col items-center gap-3">
                <div
                  class="w-20 h-20 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-black text-3xl shadow-xl shadow-indigo-500/30 ring-4 ring-white/20"
                >
                  {initials}
                </div>
                <div class="text-center">
                  <p
                    class="font-bold text-base tracking-tight truncate max-w-[200px]"
                  >
                    {displayName || "—"}
                  </p>
                  <p
                    class="text-xs text-slate-500 dark:text-slate-400 truncate max-w-[200px] mt-0.5"
                  >
                    {$user?.email || ""}
                  </p>
                </div>
              </div>
            </div>

            <!-- Info List -->
            <div class="p-4 space-y-3">
              <div
                class="flex items-center gap-3 rounded-2xl bg-gray-50 dark:bg-gray-800/60 px-4 py-3"
              >
                <div
                  class="rounded-xl bg-indigo-100 dark:bg-indigo-500/10 p-2 text-indigo-600 dark:text-indigo-400"
                >
                  <ShieldCheck size={14} />
                </div>
                <div class="min-w-0">
                  <p
                    class="text-[10px] font-bold uppercase tracking-widest text-gray-400 dark:text-gray-500"
                  >
                    Role
                  </p>
                  <p
                    class="text-sm font-bold text-gray-900 dark:text-white capitalize"
                  >
                    {$user?.role || "—"}
                  </p>
                </div>
              </div>

              {#if nickname}
                <div
                  class="flex items-center gap-3 rounded-2xl bg-gray-50 dark:bg-gray-800/60 px-4 py-3"
                >
                  <div
                    class="rounded-xl bg-purple-100 dark:bg-purple-500/10 p-2 text-purple-600 dark:text-purple-400"
                  >
                    <AtSign size={14} />
                  </div>
                  <div class="min-w-0">
                    <p
                      class="text-[10px] font-bold uppercase tracking-widest text-gray-400 dark:text-gray-500"
                    >
                      {$_("users__form_nickname")}
                    </p>
                    <p
                      class="text-sm font-bold text-gray-900 dark:text-white truncate"
                    >
                      {nickname}
                    </p>
                  </div>
                </div>
              {/if}

              {#if position}
                <div
                  class="flex items-center gap-3 rounded-2xl bg-gray-50 dark:bg-gray-800/60 px-4 py-3"
                >
                  <div
                    class="rounded-xl bg-emerald-100 dark:bg-emerald-500/10 p-2 text-emerald-600 dark:text-emerald-400"
                  >
                    <Fingerprint size={14} />
                  </div>
                  <div class="min-w-0">
                    <p
                      class="text-[10px] font-bold uppercase tracking-widest text-gray-400 dark:text-gray-500"
                    >
                      {$_("users__form_position")}
                    </p>
                    <p
                      class="text-sm font-bold text-gray-900 dark:text-white truncate"
                    >
                      {position}
                    </p>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </aside>

        <!-- Right: Form -->
        <section class="min-w-0 space-y-6 animate-fade-in">
          <!-- Header Card -->
          <div
            class="overflow-hidden rounded-[30px] border border-slate-200 bg-white shadow-[0_30px_80px_rgba(15,23,42,0.12)] dark:border-slate-800/80 dark:bg-slate-950 dark:shadow-[0_30px_80px_rgba(15,23,42,0.45)]"
          >
            <div
              class="bg-[radial-gradient(circle_at_top_left,_rgba(99,102,241,0.20),_transparent_35%),linear-gradient(135deg,_#f8fafc_0%,_#e2e8f0_48%,_#c7d2fe_100%)] px-6 py-7 dark:bg-[radial-gradient(circle_at_top_left,_rgba(99,102,241,0.28),_transparent_35%),linear-gradient(135deg,_#020617_0%,_#0f172a_48%,_#1e1b4b_100%)] sm:px-8"
            >
              <div
                class="mb-4 inline-flex rounded-2xl bg-white/70 p-3 text-slate-700 ring-1 ring-slate-200/80 backdrop-blur dark:bg-white/10 dark:text-white dark:ring-white/15"
              >
                <UserCircle2 size={22} />
              </div>
              <h2
                class="text-2xl font-bold tracking-tight text-slate-900 dark:text-white sm:text-3xl"
              >
                {$_("profileModal__title")}
              </h2>
              <p
                class="mt-2 max-w-xl text-sm leading-6 text-slate-600 dark:text-slate-300/85"
              >
                {$_("profileModal__subtitle")}
              </p>
            </div>
          </div>

          <!-- Form Card -->
          <div
            class="overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-[0_18px_45px_rgba(15,23,42,0.08)] dark:border-gray-800 dark:bg-gray-800/50"
          >
            <form on:submit|preventDefault={handleSubmit} class="p-6 space-y-6">
              {#if error}
                <div
                  class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl text-red-600 dark:text-red-400 text-sm font-medium"
                >
                  {error}
                </div>
              {/if}

              {#if saveSuccess}
                <div
                  class="p-3 bg-emerald-50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800 rounded-xl text-emerald-600 dark:text-emerald-400 text-sm font-medium"
                >
                  {$_("profileModal__save_success")}
                </div>
              {/if}

              <!-- Email (read-only) -->
              <div>
                <label
                  for="email"
                  class="flex items-center gap-1.5 text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                >
                  <Mail size={12} />
                  {$_("profileModal__email_readonly")}
                </label>
                <div class="relative">
                  <input
                    type="email"
                    id="email"
                    value={$user?.email ?? ""}
                    disabled
                    class="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-500 dark:text-gray-500 text-sm cursor-not-allowed"
                  />
                  <div
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400"
                  >
                    <Lock size={14} />
                  </div>
                </div>
              </div>

              <!-- First / Last Name -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label
                    for="firstName"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("users__form_first_name")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                    >
                      <User size={16} />
                    </div>
                    <input
                      type="text"
                      id="firstName"
                      bind:value={firstName}
                      class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                      placeholder={$_("profileModal__first_name_placeholder")}
                    />
                  </div>
                </div>
                <div>
                  <label
                    for="lastName"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("users__form_last_name")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                    >
                      <User size={16} />
                    </div>
                    <input
                      type="text"
                      id="lastName"
                      bind:value={lastName}
                      class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                      placeholder={$_("profileModal__last_name_placeholder")}
                    />
                  </div>
                </div>
              </div>

              <!-- Nickname / Position -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label
                    for="nickname"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("users__form_nickname")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                    >
                      <AtSign size={16} />
                    </div>
                    <input
                      type="text"
                      id="nickname"
                      bind:value={nickname}
                      class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                      placeholder={$_("profileModal__nickname_placeholder")}
                    />
                  </div>
                </div>
                <div>
                  <label
                    for="position"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("users__form_position")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 z-10 text-gray-400 group-focus-within:text-indigo-500 transition-colors pointer-events-none"
                    >
                      <Fingerprint size={16} />
                    </div>
                    <SearchableSelect
                      id="position"
                      bind:value={position}
                      options={positionOptions}
                      placeholder={$_("profileModal__position_placeholder")}
                      showSearch={false}
                      customClass="pl-10"
                    />
                  </div>
                </div>
              </div>

              <!-- Discord ID -->
              <div>
                <label
                  for="discordId"
                  class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                >
                  {$_("users__form_discord_id")}
                </label>
                <div class="relative group">
                  <div
                    class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                  >
                    <svg
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="currentColor"
                    >
                      <path
                        d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.196.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.419 0 1.334-.956 2.419-2.157 2.419zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.419 0 1.334-.946 2.419-2.157 2.419z"
                      />
                    </svg>
                  </div>
                  <input
                    type="text"
                    id="discordId"
                    bind:value={discordId}
                    class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                    placeholder={$_("profileModal__discord_id_placeholder")}
                  />
                </div>
              </div>

              <hr class="border-gray-100 dark:border-gray-700" />

              <!-- Password -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label
                    for="password"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("profileModal__new_password")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                    >
                      <Lock size={16} />
                    </div>
                    <input
                      type="password"
                      id="password"
                      bind:value={password}
                      class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                      placeholder="••••••••"
                    />
                  </div>
                </div>
                <div>
                  <label
                    for="confirmPassword"
                    class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5"
                  >
                    {$_("profileModal__confirm_password")}
                  </label>
                  <div class="relative group">
                    <div
                      class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-indigo-500 transition-colors"
                    >
                      <Lock size={16} />
                    </div>
                    <input
                      type="password"
                      id="confirmPassword"
                      bind:value={confirmPassword}
                      class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm"
                      placeholder="••••••••"
                    />
                  </div>
                </div>
              </div>

              <!-- Submit -->
              <div class="pt-2">
                <button
                  type="submit"
                  disabled={loading}
                  class="w-full flex items-center justify-center gap-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-400 text-white font-bold py-3 rounded-xl transition-all shadow-lg shadow-indigo-600/20 active:scale-[0.98]"
                >
                  {#if loading}
                    <div
                      class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"
                    ></div>
                  {:else}
                    <Save size={18} />
                  {/if}
                  {$_("profileModal__btn_save")}
                </button>
              </div>
            </form>
          </div>
        </section>
      </div>
    </div>
  </main>
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
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.2);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(156, 163, 175, 0.4);
  }
</style>
