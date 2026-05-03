<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { Link, X, Play, Check, Copy, AlertTriangle } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { browser } from "$app/environment";
  import { base } from "$app/paths";
  import { goto } from "$app/navigation";

  const dispatch = createEventDispatcher<{ close: void }>();

  const STORAGE_KEY = "deep-link-tester";
  const BASE = base;

  type RouteDef = {
    pattern: string;
    params: string[];
    description: string;
  };

  const routeDefinitions: RouteDef[] = [
    { pattern: "/", params: [], description: "Root → redirect to /dashboard" },
    { pattern: "/login", params: [], description: "Login page" },
    { pattern: "/create-account", params: [], description: "Create account" },
    { pattern: "/setup-password", params: [], description: "Setup password" },
    { pattern: "/dashboard", params: [], description: "Dashboard" },
    { pattern: "/profile", params: [], description: "User profile" },
    { pattern: "/settings", params: [], description: "Settings" },
    { pattern: "/settings/storage", params: [], description: "Storage settings" },
    { pattern: "/settings/users", params: [], description: "User management (admin)" },
    { pattern: "/settings/mcp", params: [], description: "MCP settings" },
    { pattern: "/workspace/:workspace_id", params: ["workspace_id"], description: "Workspace main page" },
    { pattern: "/workspace/:workspace_id/overview", params: ["workspace_id"], description: "Workspace overview" },
    { pattern: "/workspace/:workspace_id/task/:task_id", params: ["workspace_id", "task_id"], description: "Individual task" },
    { pattern: "/workspace/:workspace_id/test-cases", params: ["workspace_id"], description: "Test cases list" },
    { pattern: "/workspace/:workspace_id/test-cases/editor", params: ["workspace_id"], description: "Test case editor" },
    { pattern: "/workspace/:workspace_id/settings", params: ["workspace_id"], description: "Workspace settings" },
    { pattern: "/public/:workspace_id/test-cases", params: ["workspace_id"], description: "Public test cases" },
  ];

  let inputUrl = $state("");
  let navigationResult: { success: boolean; resolvedPath: string; message: string } | null = $state(null);
  let matchedRoute: RouteDef | null = $state(null);
  let extractedParams: Record<string, string> = $state({});
  let copied = $state(false);
  let testHistory: { url: string; timestamp: number; success: boolean }[] = $state([]);
  let customParams: Record<string, string> = $state({});

  function buildFullUrl(pattern: string): string {
    let url = pattern;
    for (const param of routeDefinitions.find(r => r.pattern === pattern)?.params || []) {
      url = url.replace(`:${param}`, customParams[param] || `<${param}>`);
    }
    return `${BASE}${url}`;
  }

  function parseInputUrl(url: string): { path: string; query: Record<string, string> } | null {
    try {
      const trimmed = url.trim();
      let fullPath = trimmed;

      if (fullPath.startsWith("http://") || fullPath.startsWith("https://")) {
        try {
          const u = new URL(fullPath);
          fullPath = u.pathname + u.search;
        } catch {
          return null;
        }
      }

      if (fullPath.startsWith(BASE)) {
        fullPath = fullPath.slice(BASE.length) || "/";
      }

      const [path, queryString] = fullPath.split("?");
      const query: Record<string, string> = {};
      if (queryString) {
        for (const pair of queryString.split("&")) {
          const [key, val] = pair.split("=");
          if (key) query[decodeURIComponent(key)] = decodeURIComponent(val || "");
        }
      }

      return { path: path || "/", query };
    } catch {
      return null;
    }
  }

  function matchRoute(path: string): { route: RouteDef; params: Record<string, string> } | null {
    const segments = path.split("/").filter(Boolean);

    for (const route of routeDefinitions) {
      const routeSegments = route.pattern.split("/").filter(Boolean);
      if (routeSegments.length !== segments.length) continue;

      const params: Record<string, string> = {};
      let match = true;

      for (let i = 0; i < routeSegments.length; i++) {
        if (routeSegments[i].startsWith(":")) {
          params[routeSegments[i].slice(1)] = segments[i];
        } else if (routeSegments[i] !== segments[i]) {
          match = false;
          break;
        }
      }

      if (match) return { route, params };
    }
    return null;
  }

  function testUrl() {
    if (!inputUrl.trim()) return;

    const parsed = parseInputUrl(inputUrl);
    if (!parsed) {
      navigationResult = { success: false, resolvedPath: inputUrl, message: "Invalid URL format" };
      matchedRoute = null;
      extractedParams = {};
      return;
    }

    const result = matchRoute(parsed.path);
    if (result) {
      matchedRoute = result.route;
      extractedParams = { ...result.params, ...parsed.query };
      navigationResult = {
        success: true,
        resolvedPath: parsed.path,
        message: `Matched: ${result.route.description}`,
      };
    } else {
      matchedRoute = null;
      extractedParams = parsed.query;
      navigationResult = {
        success: false,
        resolvedPath: parsed.path,
        message: "No matching route found — 404",
      };
    }

    testHistory = [{ url: inputUrl, timestamp: Date.now(), success: navigationResult.success }, ...testHistory].slice(0, 20);
  }

  function navigateTo() {
    if (!inputUrl.trim()) return;
    const parsed = parseInputUrl(inputUrl);
    if (!parsed) return;

    const targetPath = `${BASE}${parsed.path}`;
    const queryStr = Object.entries(extractedParams).length > 0
      ? "?" + Object.entries(extractedParams).map(([k, v]) => `${k}=${v}`).join("&")
      : "";

    goto(`${targetPath}${queryStr}`);
    handleClose();
  }

  function setQuickRoute(pattern: string) {
    inputUrl = buildFullUrl(pattern);
    testUrl();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleClose();
    if (e.key === "Enter" && !e.shiftKey) testUrl();
  }

  function handleClose() {
    dispatch("close");
  }

  function handleCopy() {
    navigator.clipboard.writeText(inputUrl);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function copyResolvedPath() {
    if (navigationResult) {
      navigator.clipboard.writeText(`${BASE}${navigationResult.resolvedPath}`);
    }
  }

  onMount(() => {
    if (browser) {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        try {
          const data = JSON.parse(saved);
          inputUrl = data.inputUrl || "";
          testHistory = data.testHistory || [];
          customParams = data.customParams || {};
        } catch {}
      }
    }
  });

  $effect(() => {
    if (browser) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ inputUrl, testHistory, customParams }));
    }
  });
</script>

{#if browser}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-5xl w-full h-[85vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
            <Link class="text-blue-600 dark:text-blue-400" size={20} />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {$_('deepLinkTester__title') || 'Deep Link Tester'}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {$_('deepLinkTester__subtitle') || 'Test deep link URLs without rebuilding'}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={handleCopy}
            class="p-2 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all"
            title={$_('deepLinkTester__copy') || 'Copy URL'}
          >
            {#if copied}
              <Check size={18} class="text-green-500" />
            {:else}
              <Copy size={18} />
            {/if}
          </button>
          <button
            onclick={handleClose}
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X size={20} />
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        <!-- URL Input -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('deepLinkTester__url_input') || 'Deep Link URL'}
          </label>
          <div class="flex items-center gap-2">
            <div class="flex-1 flex items-center bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl overflow-hidden focus-within:border-blue-400 dark:focus-within:border-blue-600 transition-colors">
              <span class="px-3 text-blue-500 font-mono font-bold text-sm shrink-0">
                <Link size={14} />
              </span>
              <input
                type="text"
                bind:value={inputUrl}
                class="flex-1 bg-transparent py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none placeholder-gray-400"
                placeholder="{BASE}/workspace/abc123/task/456?room=xyz"
              />
            </div>
            <button
              onclick={testUrl}
              class="px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-xl text-sm font-bold flex items-center gap-2 transition-colors shrink-0"
            >
              <Play size={14} />
              {$_('deepLinkTester__test') || 'Test'}
            </button>
          </div>
        </div>

        <!-- Result -->
        {#if navigationResult}
          <div class="rounded-xl border {navigationResult.success
            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
            : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'} px-4 py-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                {#if navigationResult.success}
                  <Check size={16} class="text-green-600 dark:text-green-400 shrink-0" />
                {:else}
                  <AlertTriangle size={16} class="text-red-600 dark:text-red-400 shrink-0" />
                {/if}
                <span class="text-sm font-semibold {navigationResult.success
                  ? 'text-green-700 dark:text-green-300'
                  : 'text-red-700 dark:text-red-300'}">
                  {navigationResult.message}
                </span>
              </div>
              {#if navigationResult.success}
                <div class="flex items-center gap-2">
                  <button
                    onclick={navigateTo}
                    class="px-3 py-1 bg-green-600 hover:bg-green-700 text-white rounded-lg text-xs font-bold flex items-center gap-1 transition-colors"
                  >
                    <Play size={10} />
                    {$_('deepLinkTester__navigate') || 'Go'}
                  </button>
                  <button
                    onclick={copyResolvedPath}
                    class="px-3 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-xs font-bold transition-colors"
                  >
                    <Copy size={10} />
                  </button>
                </div>
              {/if}
            </div>
            <div class="mt-2 font-mono text-xs text-gray-600 dark:text-gray-400 break-all">
              {navigationResult.resolvedPath}
            </div>
          </div>
        {/if}

        <!-- Matched Route Info -->
        {#if matchedRoute}
          <div>
            <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
              {$_('deepLinkTester__matched_route') || 'Matched Route'}
            </label>
            <div class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 space-y-2">
              <div class="flex items-center gap-2">
                <code class="text-sm font-bold text-blue-600 dark:text-blue-400">{matchedRoute.pattern}</code>
                <span class="text-xs text-gray-400">— {matchedRoute.description}</span>
              </div>
              {#if Object.keys(extractedParams).length > 0}
                <div class="flex flex-wrap gap-2 pt-1">
                  {#each Object.entries(extractedParams) as [key, value]}
                    <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-800">
                      <span class="text-blue-400 dark:text-blue-500">{key}=</span>"{value}"
                    </span>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Route Reference -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('deepLinkTester__quick_routes') || 'Quick Routes'}
          </label>
          <div class="space-y-1 max-h-[250px] overflow-y-auto rounded-xl border border-gray-200 dark:border-gray-700">
            {#each routeDefinitions as route}
              <button
                onclick={() => setQuickRoute(route.pattern)}
                class="w-full flex items-center gap-3 px-3 py-2 text-left hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors group"
              >
                <code class="text-xs font-mono text-blue-600 dark:text-blue-400 shrink-0 w-[280px] truncate">{buildFullUrl(route.pattern)}</code>
                <span class="text-[10px] text-gray-400 dark:text-gray-500 flex-1 truncate">{route.description}</span>
                {#if route.params.length > 0}
                  <span class="text-[10px] text-orange-500 bg-orange-100 dark:bg-orange-900/30 px-1.5 py-0.5 rounded font-bold shrink-0">
                    {route.params.length} param{route.params.length > 1 ? 's' : ''}
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <!-- Custom Params -->
        <div>
          <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {$_('deepLinkTester__custom_params') || 'Custom Param Values'}
          </label>
          <div class="flex flex-wrap gap-2">
            {#each ["workspace_id", "task_id"] as param}
              <div class="flex items-center gap-1.5 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-2.5 py-1.5">
                <span class="text-[10px] font-bold text-gray-400 uppercase">{param}:</span>
                <input
                  type="text"
                  value={customParams[param] || ""}
                  oninput={(e) => { customParams[param] = (e.target as HTMLInputElement).value; }}
                  class="bg-transparent text-xs font-mono text-gray-800 dark:text-gray-200 outline-none w-24 placeholder-gray-400"
                  placeholder="value"
                />
              </div>
            {/each}
          </div>
        </div>

        <!-- History -->
        {#if testHistory.length > 0}
          <div>
            <label class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
              {$_('deepLinkTester__history') || 'History'} ({testHistory.length})
            </label>
            <div class="space-y-1 max-h-40 overflow-y-auto">
              {#each testHistory as entry}
                <button
                  onclick={() => { inputUrl = entry.url; testUrl(); }}
                  class="w-full flex items-center gap-2 px-3 py-1.5 rounded-lg text-left hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group"
                >
                  <span class="inline-block w-2 h-2 rounded-full shrink-0 {entry.success ? 'bg-green-500' : 'bg-red-500'}"></span>
                  <code class="text-xs font-mono text-gray-600 dark:text-gray-400 truncate flex-1">{entry.url}</code>
                  <span class="text-[10px] text-gray-400 shrink-0">
                    {new Date(entry.timestamp).toLocaleTimeString()}
                  </span>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
        <div class="flex items-center gap-2">
          <span class="inline-block w-2 h-2 rounded-full {navigationResult ? (navigationResult.success ? 'bg-green-500' : 'bg-red-500') : 'bg-gray-400'}"></span>
          <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
            {navigationResult
              ? (navigationResult.success ? $_('deepLinkTester__valid') || 'Valid route' : $_('deepLinkTester__invalid') || 'Invalid route')
              : $_('deepLinkTester__ready') || 'Ready'}
          </span>
        </div>
        <div class="text-[10px] text-gray-400 font-mono">
          {routeDefinitions.length} routes · base: {BASE}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes modal-in {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(-10px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .animate-modal-in {
    animation: modal-in 0.2s ease-out;
  }
</style>
