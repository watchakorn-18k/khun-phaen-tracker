<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { onMount } from "svelte";
  import { user } from "$lib/stores/auth";
  import {
    currentWorkspaceName,
    currentWorkspaceColor,
    currentWorkspaceIcon,
    clearWorkspaceId,
    setWorkspaceId,
    MY_TASKS_WORKSPACE_ID,
  } from "$lib/stores/workspace";
  import { theme } from "$lib/stores/theme";
  import { locale, changeLocale } from "$lib/i18n";
  import { _ } from "svelte-i18n";
  import { api } from "$lib/apis";
  import {
    ListTodo,
    Settings,
    Sun,
    Moon,
    Globe,
    LogOut,
    ChevronDown,
    Target,
    Plus,
    Check,
    LayoutDashboard,
  } from "lucide-svelte";
  import SidebarUtilityTools from "$lib/components/SidebarUtilityTools.svelte";

  interface Workspace {
    id: string;
    name: string;
    short_name?: string;
    room_code: string;
    owner_id: string;
    color?: string;
    icon?: string;
  }

  let showWorkspaceSwitcher = false;
  let showLanguageDropdown = false;
  let showAccountDropdown = false;
  let workspaceList: Workspace[] = [];

  $: workspaceId = $page.params.workspace_id;
  $: isMyTasksWorkspace = workspaceId === MY_TASKS_WORKSPACE_ID;
  $: workspaceName =
    $currentWorkspaceName || (isMyTasksWorkspace ? "Focus Hub" : "Workspace");
  $: workspaceColor = $currentWorkspaceColor || "#6366f1";
  $: workspaceIcon = $currentWorkspaceIcon;

  $: currentPath = $page.url.pathname;

  function isActive(path: string): boolean {
    return currentPath === path || currentPath.startsWith(path + "?");
  }

  async function loadWorkspaces() {
    try {
      const res = await api.workspaces.getList();
      if (res.ok) {
        const data = await res.json();
        workspaceList = data.workspaces || [];
      }
    } catch {}
  }

  function workspaceUrl(ws: Workspace) {
    return `${base}/workspace/${ws.id}?room=${ws.room_code}`;
  }

  function prepareWorkspaceSwitch(ws: Workspace) {
    closeAllDropdowns();
    setWorkspaceId(ws.id, ws.name, ws.owner_id, ws.color, ws.icon, ws.short_name);
    localStorage.setItem("sync-room-code", ws.room_code);
    localStorage.setItem(
      "backend-server-url",
      import.meta.env.VITE_SERVER_URL || "http://127.0.0.1:3002",
    );
  }

  async function handleLogout() {
    try {
      await api.auth.logout();
      document.cookie = "_khun_ph_token=; path=/; max-age=0; samesite=Lax";
      localStorage.removeItem("user_email");
      localStorage.removeItem("sync-room-code");
      localStorage.removeItem("backend-server-url");
      clearWorkspaceId();
      user.set(null);
      goto(`${base}/login`);
    } catch (e) {
      console.error("Logout failed:", e);
    }
  }

  function getInitials(name: string): string {
    return name.trim().charAt(0).toUpperCase();
  }

  function closeAllDropdowns() {
    showWorkspaceSwitcher = false;
    showLanguageDropdown = false;
    showAccountDropdown = false;
  }

  onMount(() => {
    loadWorkspaces();
  });

  $: workspaceHref = isMyTasksWorkspace
    ? `${base}/workspace/${MY_TASKS_WORKSPACE_ID}${$page.url.search}`
    : `${base}/workspace/${workspaceId}${$page.url.search}`;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if showWorkspaceSwitcher || showLanguageDropdown || showAccountDropdown}
  <div class="fixed inset-0 z-10" on:click={closeAllDropdowns}></div>
{/if}

<div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
  <!-- Sidebar -->
  <aside
    class="w-64 flex flex-col shrink-0 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 z-20"
  >
    <!-- Workspace Switcher -->
    <div class="p-3 border-b border-gray-200 dark:border-gray-800">
      <div class="relative">
        <button
          type="button"
          on:click|stopPropagation={() => {
            showWorkspaceSwitcher = !showWorkspaceSwitcher;
            showLanguageDropdown = false;
            showAccountDropdown = false;
          }}
          class="w-full flex items-center gap-2 px-2 py-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors text-left"
        >
          <!-- Workspace Avatar -->
          <div
            class="w-6 h-6 rounded-md flex items-center justify-center text-white text-xs font-bold shrink-0"
            style="background-color: {workspaceColor}"
          >
            {#if workspaceIcon === "Target"}
              <Target size={14} />
            {:else}
              {getInitials(workspaceName)}
            {/if}
          </div>
          <span class="flex-1 truncate text-sm font-medium text-gray-900 dark:text-gray-100">
            {workspaceName}
          </span>
          <ChevronDown size={14} class="text-gray-500 shrink-0" />
        </button>

        {#if showWorkspaceSwitcher}
          <div
            class="absolute left-0 top-full mt-1 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-40"
          >
            <!-- Workspace list label -->
            <p class="px-3 pt-2 pb-1 text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
              Workspaces
            </p>
            <div class="px-1 pb-1">
              {#each workspaceList as ws}
                <a
                  href={workspaceUrl(ws)}
                  on:click={() => prepareWorkspaceSwitch(ws)}
                  class="w-full flex items-center gap-2.5 px-2 py-2 rounded-md text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
                >
                  <div
                    class="w-5 h-5 rounded flex items-center justify-center text-white text-[10px] font-bold shrink-0"
                    style="background-color: {ws.color || '#6366f1'}"
                  >
                    {getInitials(ws.name)}
                  </div>
                  <span class="flex-1 truncate">{ws.name}</span>
                  {#if ws.id === workspaceId}
                    <Check size={13} class="text-primary shrink-0" />
                  {/if}
                </a>
              {/each}
            </div>

            <div class="border-t border-gray-100 dark:border-gray-700 px-1 py-1">
              <a
                href="{base}/dashboard"
                on:click={closeAllDropdowns}
                class="flex items-center gap-2 px-2 py-2 rounded-md text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              >
                <Plus size={14} />
                <span>Create Workspace</span>
              </a>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 overflow-y-auto p-2 space-y-4">
      <div>
        <div class="px-2 py-1 text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
          Personal
        </div>
        <div class="mt-1 space-y-0.5">
          <a
            href="{base}/dashboard"
            class="flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm transition-colors {isActive(
              `${base}/dashboard`,
            )
              ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white font-medium'
              : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white'}"
          >
            <LayoutDashboard size={16} class="shrink-0" />
            <span>Dashboard</span>
          </a>
          <a
            href="{base}/workspace/{MY_TASKS_WORKSPACE_ID}"
            class="flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm transition-colors {isMyTasksWorkspace
              ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white font-medium'
              : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white'}"
          >
            <Target size={16} class="shrink-0" />
            <span>My Tasks</span>
          </a>
        </div>

        <div class="mt-4">
          <SidebarUtilityTools />
        </div>
      </div>

      <!-- Workspace -->
      {#if !isMyTasksWorkspace}
        <div>
          <div class="px-2 py-1 text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
            Workspace
          </div>
          <div class="mt-1 space-y-0.5">
            <a
              href={workspaceHref}
              class="flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm transition-colors {isActive(
                `${base}/workspace/${workspaceId}`,
              )
                ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white font-medium'
                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white'}"
            >
              <ListTodo size={16} class="shrink-0" />
              <span>Issues</span>
            </a>
          </div>
        </div>
      {/if}

      <!-- Configure -->
      {#if $user?.role === "admin"}
        <div>
          <div class="px-2 py-1 text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
            Configure
          </div>
          <div class="mt-1 space-y-0.5">
            <a
              href="{base}/settings"
              class="flex items-center gap-2.5 px-2 py-1.5 rounded-md text-sm transition-colors {isActive(
                `${base}/settings`,
              )
                ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white font-medium'
                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white'}"
            >
              <Settings size={16} class="shrink-0" />
              <span>Settings</span>
            </a>
          </div>
        </div>
      {/if}
    </nav>

    <!-- Footer -->
    <div
      class="p-2 border-t border-gray-200 dark:border-gray-800 flex items-center gap-1"
    >
      <!-- Theme Toggle -->
      <button
        on:click={() => theme.toggle()}
        class="p-2 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title={$_("layout__change_theme")}
      >
        {#if $theme === "light"}
          <Sun size={16} />
        {:else}
          <Moon size={16} />
        {/if}
      </button>

      <!-- Language Switcher -->
      <div class="relative">
        <button
          type="button"
          on:click|stopPropagation={() => {
            showLanguageDropdown = !showLanguageDropdown;
            showWorkspaceSwitcher = false;
            showAccountDropdown = false;
          }}
          class="p-2 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1"
          title={$_("layout__change_language")}
        >
          <Globe size={16} />
          <span class="text-xs font-medium uppercase">{$locale}</span>
        </button>

        {#if showLanguageDropdown}
          <div
            class="absolute bottom-full left-0 mb-1 w-32 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-40"
          >
            <button
              class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2 {$locale ===
              'th'
                ? 'text-primary font-medium'
                : ''}"
              on:click={() => {
                changeLocale("th");
                showLanguageDropdown = false;
              }}
            >
              <span>🇹🇭</span> ไทย
            </button>
            <button
              class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2 {$locale ===
              'en'
                ? 'text-primary font-medium'
                : ''}"
              on:click={() => {
                changeLocale("en");
                showLanguageDropdown = false;
              }}
            >
              <span>🇺🇸</span> English
            </button>
          </div>
        {/if}
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- User Avatar -->
      {#if $user}
        <div class="relative">
          <button
            type="button"
            on:click|stopPropagation={() => {
              showAccountDropdown = !showAccountDropdown;
              showWorkspaceSwitcher = false;
              showLanguageDropdown = false;
            }}
            class="w-7 h-7 rounded-full bg-indigo-500 flex items-center justify-center text-white font-bold text-xs hover:ring-2 hover:ring-indigo-400 transition-all"
            title={$user.profile?.nickname || $user.email}
          >
            {getInitials($user.profile?.nickname || $user.email.split("@")[0])}
          </button>

          {#if showAccountDropdown}
            <div
              class="absolute bottom-full right-0 mb-2 w-56 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-40"
            >
              <div class="px-3 py-2.5 border-b border-gray-100 dark:border-gray-700">
                <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {$user.profile?.nickname || $user.email.split("@")[0]}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
                  {$user.email}
                </p>
              </div>
              <div class="px-1 py-1">
                <button
                  on:click={() => {
                    closeAllDropdowns();
                    handleLogout();
                  }}
                  class="w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                >
                  <LogOut size={14} />
                  <span>{$_("layout__logout")}</span>
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 overflow-y-auto min-w-0">
    {#key workspaceId}
      <slot />
    {/key}
  </main>
</div>
