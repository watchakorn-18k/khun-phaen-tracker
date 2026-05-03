<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { Globe, X, Copy, Check, Plus, Trash2, Send, Clock, ChevronDown, Loader2, Zap, Cable } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();

  const STORAGE_KEY = "api-tester";

  type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";
  type KeyValue = { key: string; value: string; enabled: boolean };
  type AuthType = "none" | "bearer" | "basic" | "apikey";
  type BodyType = "none" | "json" | "form" | "raw" | "binary";
  type Tab = "headers" | "body" | "auth" | "params";
  type WsState = "disconnected" | "connecting" | "connected" | "error";

  const methods: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
  const methodColors: Record<HttpMethod, string> = {
    GET: "text-green-600 dark:text-green-400",
    POST: "text-amber-600 dark:text-amber-400",
    PUT: "text-blue-600 dark:text-blue-400",
    DELETE: "text-red-600 dark:text-red-400",
    PATCH: "text-purple-600 dark:text-purple-400",
    HEAD: "text-teal-600 dark:text-teal-400",
    OPTIONS: "text-gray-600 dark:text-gray-400",
  };
  const methodBgColors: Record<HttpMethod, string> = {
    GET: "bg-green-100 dark:bg-green-900/30",
    POST: "bg-amber-100 dark:bg-amber-900/30",
    PUT: "bg-blue-100 dark:bg-blue-900/30",
    DELETE: "bg-red-100 dark:bg-red-900/30",
    PATCH: "bg-purple-100 dark:bg-purple-900/30",
    HEAD: "bg-teal-100 dark:bg-teal-900/30",
    OPTIONS: "bg-gray-100 dark:bg-gray-700",
  };

  let method = $state<HttpMethod>("GET");
  let url = $state("");
  let activeTab = $state<Tab>("headers");
  let headers = $state<KeyValue[]>([{ key: "Content-Type", value: "application/json", enabled: true }]);
  let queryParams = $state<KeyValue[]>([]);
  let bodyType = $state<BodyType>("json");
  let bodyContent = $state('{\n  \n}');
  let authType = $state<AuthType>("none");
  let bearerToken = $state("");
  let basicUser = $state("");
  let basicPass = $state("");
  let apiKeyHeader = $state("");
  let apiKeyValue = $state("");

  let loading = $state(false);
  let responseStatus = $state(0);
  let responseStatusText = $state("");
  let responseTime = $state(0);
  let responseSize = $state(0);
  let responseHeaders = $state<{ key: string; value: string }[]>([]);
  let responseBody = $state("");
  let responseError = $state("");
  let responseTab = $state<"body" | "headers">("body");
  let copied = $state(false);

  // WebSocket state
  let wsMode = $state(false);
  let wsUrl = $state("");
  let wsState = $state<WsState>("disconnected");
  let wsMessage = $state("");
  let wsMessages = $state<{ direction: "sent" | "received"; content: string; time: number }[]>([]);
  let ws: WebSocket | null = null;

  let showMethodDropdown = $state(false);

  function getFullUrl(): string {
    if (!url) return "";
    const params = queryParams.filter((p) => p.enabled && p.key);
    if (params.length === 0) return url;
    const separator = url.includes("?") ? "&" : "?";
    const qs = params.map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`).join("&");
    return url + separator + qs;
  }

  async function sendRequest() {
    if (!url.trim()) return;
    loading = true;
    responseError = "";
    responseBody = "";
    responseHeaders = [];
    responseStatus = 0;
    responseStatusText = "";

    const fullUrl = getFullUrl();
    const start = performance.now();

    try {
      const fetchHeaders: Record<string, string> = {};
      for (const h of headers) {
        if (h.enabled && h.key.trim()) {
          fetchHeaders[h.key.trim()] = h.value;
        }
      }

      if (authType === "bearer" && bearerToken) {
        fetchHeaders["Authorization"] = `Bearer ${bearerToken}`;
      } else if (authType === "basic" && basicUser) {
        fetchHeaders["Authorization"] = `Basic ${btoa(`${basicUser}:${basicPass}`)}`;
      } else if (authType === "apikey" && apiKeyHeader) {
        fetchHeaders[apiKeyHeader] = apiKeyValue;
      }

      const options: RequestInit = {
        method,
        headers: fetchHeaders,
      };

      if (!["GET", "HEAD"].includes(method) && bodyType !== "none") {
        if (bodyType === "json") {
          options.body = bodyContent;
          if (!fetchHeaders["Content-Type"]) {
            fetchHeaders["Content-Type"] = "application/json";
          }
        } else if (bodyType === "form") {
          const formData = new FormData();
          try {
            const parsed = JSON.parse(bodyContent);
            if (typeof parsed === "object" && parsed !== null) {
              for (const [k, v] of Object.entries(parsed)) {
                formData.append(k, String(v));
              }
            }
          } catch {
            formData.append("data", bodyContent);
          }
          options.body = formData;
        } else if (bodyType === "raw") {
          options.body = bodyContent;
        }
      }

      const resp = await fetch(fullUrl, options);
      const end = performance.now();
      responseTime = Math.round(end - start);
      responseStatus = resp.status;
      responseStatusText = resp.statusText;

      const respHeaders: { key: string; value: string }[] = [];
      resp.headers.forEach((v, k) => respHeaders.push({ key: k, value: v }));
      responseHeaders = respHeaders;

      const text = await resp.text();
      responseBody = text;
      responseSize = new Blob([text]).size;
    } catch (e: any) {
      responseError = e.message || "Request failed";
    } finally {
      loading = false;
    }
  }

  function formatBody(body: string): string {
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
    }
  }

  function statusColor(status: number): string {
    if (status >= 200 && status < 300) return "text-green-600 dark:text-green-400";
    if (status >= 300 && status < 400) return "text-blue-600 dark:text-blue-400";
    if (status >= 400 && status < 500) return "text-amber-600 dark:text-amber-400";
    return "text-red-600 dark:text-red-400";
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function addHeader() {
    headers = [...headers, { key: "", value: "", enabled: true }];
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index);
  }

  function addParam() {
    queryParams = [...queryParams, { key: "", value: "", enabled: true }];
  }

  function removeParam(index: number) {
    queryParams = queryParams.filter((_, i) => i !== index);
  }

  function handleCopy() {
    navigator.clipboard.writeText(responseBody);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showMethodDropdown) {
        showMethodDropdown = false;
        return;
      }
      handleClose();
    }
  }

  function handleClose() {
    dispatch("close");
  }

  // WebSocket
  function connectWs() {
    if (!wsUrl.trim()) return;
    wsState = "connecting";
    try {
      ws = new WebSocket(wsUrl);
      ws.onopen = () => {
        wsState = "connected";
      };
      ws.onmessage = (event) => {
        wsMessages = [...wsMessages, { direction: "received", content: event.data, time: Date.now() }];
      };
      ws.onerror = () => {
        wsState = "error";
      };
      ws.onclose = () => {
        wsState = "disconnected";
        ws = null;
      };
    } catch {
      wsState = "error";
    }
  }

  function disconnectWs() {
    if (ws) {
      ws.close();
      ws = null;
    }
    wsState = "disconnected";
  }

  function sendWsMessage() {
    if (!ws || ws.readyState !== WebSocket.OPEN || !wsMessage.trim()) return;
    ws.send(wsMessage);
    wsMessages = [...wsMessages, { direction: "sent", content: wsMessage, time: Date.now() }];
    wsMessage = "";
  }

  function clearWsMessages() {
    wsMessages = [];
  }

  const commonHeaders = [
    { key: "Content-Type", value: "application/json" },
    { key: "Accept", value: "application/json" },
    { key: "Authorization", value: "Bearer " },
  ];

  function addCommonHeader(h: { key: string; value: string }) {
    headers = [...headers, { key: h.key, value: h.value, enabled: true }];
  }

  onMount(() => {
    if (browser) {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        try {
          const d = JSON.parse(saved);
          method = d.method || "GET";
          url = d.url || "";
          headers = d.headers || headers;
          queryParams = d.queryParams || [];
          bodyType = d.bodyType || "json";
          bodyContent = d.bodyContent || bodyContent;
          authType = d.authType || "none";
          bearerToken = d.bearerToken || "";
          basicUser = d.basicUser || "";
          basicPass = d.basicPass || "";
          apiKeyHeader = d.apiKeyHeader || "";
          apiKeyValue = d.apiKeyValue || "";
        } catch {}
      }
    }
  });

  $effect(() => {
    if (browser) {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          method,
          url,
          headers,
          queryParams,
          bodyType,
          bodyContent,
          authType,
          bearerToken,
          basicUser,
          basicPass,
          apiKeyHeader,
          apiKeyValue,
        })
      );
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
      class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-6xl w-full h-[90vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-orange-100 dark:bg-orange-900/30 rounded-lg">
            {#if wsMode}
              <Cable class="text-orange-600 dark:text-orange-400" size={20} />
            {:else}
              <Globe class="text-orange-600 dark:text-orange-400" size={20} />
            {/if}
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {$_('apiTester__title') || 'API Tester'}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {wsMode
                ? ($_('apiTester__ws_subtitle') || 'WebSocket client')
                : ($_('apiTester__subtitle') || 'Test HTTP requests like a lightweight Postman')}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onclick={() => { wsMode = !wsMode; }}
            class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all {wsMode
              ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400 border border-orange-300 dark:border-orange-700'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 border border-transparent hover:border-gray-300 dark:hover:border-gray-600'}"
          >
            {wsMode ? 'HTTP' : 'WebSocket'}
          </button>
          <button
            onclick={handleClose}
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X size={20} />
          </button>
        </div>
      </div>

      <!-- HTTP Mode -->
      {#if !wsMode}
        <!-- Request Bar -->
        <div class="px-5 py-3 border-b border-gray-100 dark:border-gray-700 shrink-0">
          <div class="flex items-center gap-2">
            <!-- Method Selector -->
            <div class="relative">
              <button
                onclick={() => (showMethodDropdown = !showMethodDropdown)}
                class="flex items-center gap-1.5 px-3 py-2.5 rounded-xl text-sm font-bold {methodBgColors[method]} {methodColors[method]} border border-gray-200 dark:border-gray-700 hover:opacity-80 transition-all min-w-[100px]"
              >
                {method}
                <ChevronDown size={14} />
              </button>
              {#if showMethodDropdown}
                <div class="absolute top-full left-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg z-10 py-1 min-w-[120px]">
                  {#each methods as m}
                    <button
                      onclick={() => { method = m; showMethodDropdown = false; }}
                      class="w-full px-4 py-2 text-left text-sm font-bold {methodColors[m]} hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                    >
                      {m}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- URL Input -->
            <input
              type="text"
              bind:value={url}
              class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
              placeholder={$_('apiTester__url_placeholder') || 'https://api.example.com/endpoint'}
              onkeydown={(e) => { if (e.key === "Enter") sendRequest(); }}
            />

            <!-- Send Button -->
            <button
              onclick={sendRequest}
              disabled={loading || !url.trim()}
              class="flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-bold text-white bg-orange-600 hover:bg-orange-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm"
            >
              {#if loading}
                <Loader2 size={16} class="animate-spin" />
              {:else}
                <Send size={16} />
              {/if}
              {$_('apiTester__send') || 'Send'}
            </button>
          </div>
        </div>

        <!-- Tabs -->
        <div class="px-5 border-b border-gray-100 dark:border-gray-700 shrink-0">
          <div class="flex items-center gap-0">
            {#each [
              { tab: "params" as Tab, label: $_('apiTester__params') || 'Params', count: queryParams.filter((p) => p.key).length },
              { tab: "headers" as Tab, label: $_('apiTester__headers') || 'Headers', count: headers.filter((h) => h.key).length },
              { tab: "body" as Tab, label: $_('apiTester__body') || 'Body' },
              { tab: "auth" as Tab, label: $_('apiTester__auth') || 'Auth' },
            ] as t}
              <button
                onclick={() => (activeTab = t.tab)}
                class="px-4 py-2.5 text-xs font-bold uppercase tracking-wider transition-all border-b-2 {activeTab === t.tab
                  ? 'text-orange-600 dark:text-orange-400 border-orange-600 dark:border-orange-400'
                  : 'text-gray-500 dark:text-gray-400 border-transparent hover:text-gray-700 dark:hover:text-gray-300'}"
              >
                {t.label}
                {#if 'count' in t && t.count > 0}
                  <span class="ml-1.5 px-1.5 py-0.5 rounded-md text-[10px] bg-gray-100 dark:bg-gray-700">{t.count}</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto p-5 space-y-4">
          <!-- Params Tab -->
          {#if activeTab === "params"}
            <div class="space-y-2">
              {#each queryParams as param, i}
                <div class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={param.enabled}
                    onchange={() => { queryParams[i].enabled = !queryParams[i].enabled; queryParams = [...queryParams]; }}
                    class="rounded border-gray-300 dark:border-gray-600 accent-orange-600"
                  />
                  <input
                    type="text"
                    value={param.key}
                    oninput={(e) => { queryParams[i].key = (e.target as HTMLInputElement).value; queryParams = [...queryParams]; }}
                    class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="key"
                  />
                  <input
                    type="text"
                    value={param.value}
                    oninput={(e) => { queryParams[i].value = (e.target as HTMLInputElement).value; queryParams = [...queryParams]; }}
                    class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="value"
                  />
                  <button
                    onclick={() => removeParam(i)}
                    class="p-2 text-gray-400 hover:text-red-500 dark:hover:text-red-400 transition-colors"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
              <button
                onclick={addParam}
                class="flex items-center gap-1.5 text-xs font-bold text-orange-600 dark:text-orange-400 hover:opacity-80 transition-all"
              >
                <Plus size={14} />
                {$_('apiTester__add_param') || 'Add parameter'}
              </button>
            </div>
          {/if}

          <!-- Headers Tab -->
          {#if activeTab === "headers"}
            <div class="space-y-2">
              <!-- Common Headers -->
              <div class="flex flex-wrap gap-2 mb-3">
                {#each commonHeaders as ch}
                  <button
                    onclick={() => addCommonHeader(ch)}
                    class="px-2.5 py-1 rounded-lg text-[11px] font-semibold bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-orange-100 dark:hover:bg-orange-900/30 hover:text-orange-600 dark:hover:text-orange-400 border border-transparent hover:border-orange-300 dark:hover:border-orange-700 transition-all"
                  >
                    + {ch.key}
                  </button>
                {/each}
              </div>
              {#each headers as header, i}
                <div class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={header.enabled}
                    onchange={() => { headers[i].enabled = !headers[i].enabled; headers = [...headers]; }}
                    class="rounded border-gray-300 dark:border-gray-600 accent-orange-600"
                  />
                  <input
                    type="text"
                    value={header.key}
                    oninput={(e) => { headers[i].key = (e.target as HTMLInputElement).value; headers = [...headers]; }}
                    class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="Header name"
                  />
                  <input
                    type="text"
                    value={header.value}
                    oninput={(e) => { headers[i].value = (e.target as HTMLInputElement).value; headers = [...headers]; }}
                    class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="Value"
                  />
                  <button
                    onclick={() => removeHeader(i)}
                    class="p-2 text-gray-400 hover:text-red-500 dark:hover:text-red-400 transition-colors"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
              <button
                onclick={addHeader}
                class="flex items-center gap-1.5 text-xs font-bold text-orange-600 dark:text-orange-400 hover:opacity-80 transition-all"
              >
                <Plus size={14} />
                {$_('apiTester__add_header') || 'Add header'}
              </button>
            </div>
          {/if}

          <!-- Body Tab -->
          {#if activeTab === "body"}
            <div class="space-y-3">
              <div class="flex items-center gap-2">
                {#each ["none", "json", "form", "raw"] as bt}
                  <button
                    onclick={() => (bodyType = bt as BodyType)}
                    class="px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition-all {bodyType === bt
                      ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400 border border-orange-300 dark:border-orange-700'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 border border-transparent hover:border-gray-300 dark:hover:border-gray-600'}"
                  >
                    {bt}
                  </button>
                {/each}
              </div>
              {#if bodyType !== "none"}
                <textarea
                  bind:value={bodyContent}
                  class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none resize-y min-h-[200px] focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                  placeholder={$_('apiTester__body_placeholder') || 'Request body...'}
                  spellcheck="false"
                ></textarea>
              {:else}
                <div class="text-sm text-gray-400 dark:text-gray-500 text-center py-8">
                  {$_('apiTester__no_body') || 'No body for this request'}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Auth Tab -->
          {#if activeTab === "auth"}
            <div class="space-y-3">
              <div class="flex items-center gap-2 flex-wrap">
                {#each [
                  { type: "none" as AuthType, label: $_('apiTester__auth_none') || 'None' },
                  { type: "bearer" as AuthType, label: "Bearer Token" },
                  { type: "basic" as AuthType, label: "Basic Auth" },
                  { type: "apikey" as AuthType, label: "API Key" },
                ] as a}
                  <button
                    onclick={() => (authType = a.type)}
                    class="px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition-all {authType === a.type
                      ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400 border border-orange-300 dark:border-orange-700'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 border border-transparent hover:border-gray-300 dark:hover:border-gray-600'}"
                  >
                    {a.label}
                  </button>
                {/each}
              </div>

              {#if authType === "bearer"}
                <input
                  type="text"
                  bind:value={bearerToken}
                  class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                  placeholder="Token..."
                />
              {:else if authType === "basic"}
                <div class="grid grid-cols-2 gap-3">
                  <input
                    type="text"
                    bind:value={basicUser}
                    class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder={$_('apiTester__username') || 'Username'}
                  />
                  <input
                    type="password"
                    bind:value={basicPass}
                    class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder={$_('apiTester__password') || 'Password'}
                  />
                </div>
              {:else if authType === "apikey"}
                <div class="grid grid-cols-2 gap-3">
                  <input
                    type="text"
                    bind:value={apiKeyHeader}
                    class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="Header name (e.g. X-API-Key)"
                  />
                  <input
                    type="text"
                    bind:value={apiKeyValue}
                    class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                    placeholder="Value"
                  />
                </div>
              {:else}
                <div class="text-sm text-gray-400 dark:text-gray-500 text-center py-8">
                  {$_('apiTester__no_auth') || 'No authentication for this request'}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Response Section -->
          {#if responseStatus || responseError}
            <div class="border-t border-gray-200 dark:border-gray-700 pt-4 mt-4">
              <div class="flex items-center justify-between mb-3">
                <h4 class="text-xs font-black text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  {$_('apiTester__response') || 'Response'}
                </h4>
                {#if responseStatus}
                  <div class="flex items-center gap-3 text-xs">
                    <span class="font-bold {statusColor(responseStatus)}">{responseStatus} {responseStatusText}</span>
                    <span class="text-gray-400 flex items-center gap-1"><Clock size={12} />{responseTime}ms</span>
                    <span class="text-gray-400">{formatSize(responseSize)}</span>
                  </div>
                {/if}
              </div>

              {#if responseError}
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono">
                  {responseError}
                </div>
              {:else}
                <!-- Response Tabs -->
                <div class="flex items-center gap-0 mb-3">
                  <button
                    onclick={() => (responseTab = "body")}
                    class="px-3 py-1.5 text-xs font-bold uppercase tracking-wider transition-all border-b-2 {responseTab === 'body'
                      ? 'text-gray-800 dark:text-gray-200 border-gray-800 dark:border-gray-200'
                      : 'text-gray-400 border-transparent hover:text-gray-600 dark:hover:text-gray-300'}"
                  >
                    {$_('apiTester__body') || 'Body'}
                  </button>
                  <button
                    onclick={() => (responseTab = "headers")}
                    class="px-3 py-1.5 text-xs font-bold uppercase tracking-wider transition-all border-b-2 {responseTab === 'headers'
                      ? 'text-gray-800 dark:text-gray-200 border-gray-800 dark:border-gray-200'
                      : 'text-gray-400 border-transparent hover:text-gray-600 dark:hover:text-gray-300'}"
                  >
                    {$_('apiTester__headers') || 'Headers'} ({responseHeaders.length})
                  </button>
                  <div class="flex-1"></div>
                  {#if responseTab === "body"}
                    <button
                      onclick={handleCopy}
                      class="p-1.5 text-gray-400 hover:text-orange-600 dark:hover:text-orange-400 transition-colors"
                      title={$_('apiTester__copy') || 'Copy'}
                    >
                      {#if copied}
                        <Check size={14} class="text-green-500" />
                      {:else}
                        <Copy size={14} />
                      {/if}
                    </button>
                  {/if}
                </div>

                {#if responseTab === "body"}
                  <pre class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 max-h-[300px] overflow-auto whitespace-pre-wrap break-all leading-relaxed">{formatBody(responseBody)}</pre>
                {:else}
                  <div class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 space-y-1 max-h-[300px] overflow-auto">
                    {#each responseHeaders as h}
                      <div class="flex items-start gap-3 text-sm">
                        <span class="font-bold text-orange-600 dark:text-orange-400 shrink-0 min-w-[150px]">{h.key}</span>
                        <span class="text-gray-600 dark:text-gray-400 font-mono break-all">{h.value}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              {/if}
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
          <div class="flex items-center gap-2">
            <span class="inline-block w-2 h-2 rounded-full {responseError ? 'bg-red-500' : responseStatus ? (responseStatus < 400 ? 'bg-green-500' : 'bg-amber-500') : 'bg-gray-400'}"></span>
            <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
              {responseError
                ? ($_('apiTester__error') || 'Error')
                : responseStatus
                  ? `${responseStatus} ${responseStatusText}`
                  : ($_('apiTester__ready') || 'Ready')}
            </span>
          </div>
          <div class="text-[10px] text-gray-400 font-mono">
            {method} {url || '...'}
          </div>
        </div>
      {:else}
        <!-- WebSocket Mode -->
        <div class="flex-1 flex flex-col overflow-hidden">
          <!-- Connection Bar -->
          <div class="px-5 py-3 border-b border-gray-100 dark:border-gray-700 shrink-0">
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={wsUrl}
                class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                placeholder="ws://localhost:8080/ws"
                onkeydown={(e) => { if (e.key === "Enter" && wsState === "disconnected") connectWs(); }}
              />
              {#if wsState === "disconnected" || wsState === "error"}
                <button
                  onclick={connectWs}
                  disabled={!wsUrl.trim()}
                  class="flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-bold text-white bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                >
                  <Zap size={16} />
                  {$_('apiTester__ws_connect') || 'Connect'}
                </button>
              {:else}
                <button
                  onclick={disconnectWs}
                  class="flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-bold text-white bg-red-600 hover:bg-red-700 transition-all"
                >
                  <X size={16} />
                  {$_('apiTester__ws_disconnect') || 'Disconnect'}
                </button>
              {/if}
            </div>
            <div class="flex items-center gap-2 mt-2">
              <span class="inline-block w-2 h-2 rounded-full {wsState === 'connected' ? 'bg-green-500' : wsState === 'connecting' ? 'bg-amber-500 animate-pulse' : wsState === 'error' ? 'bg-red-500' : 'bg-gray-400'}"></span>
              <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
                {wsState === 'connected'
                  ? ($_('apiTester__ws_connected') || 'Connected')
                  : wsState === 'connecting'
                    ? ($_('apiTester__ws_connecting') || 'Connecting...')
                    : wsState === 'error'
                      ? ($_('apiTester__ws_error') || 'Error')
                      : ($_('apiTester__ws_disconnected') || 'Disconnected')}
              </span>
            </div>
          </div>

          <!-- Messages -->
          <div class="flex-1 overflow-y-auto p-5">
            {#if wsMessages.length === 0}
              <div class="text-sm text-gray-400 dark:text-gray-500 text-center py-12">
                {$_('apiTester__ws_no_messages') || 'No messages yet. Connect and start sending.'}
              </div>
            {:else}
              <div class="space-y-2">
                {#each wsMessages as msg}
                  <div class="flex items-start gap-2">
                    <span class="shrink-0 px-1.5 py-0.5 rounded text-[10px] font-bold {msg.direction === 'sent'
                      ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
                      : 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400'}">
                      {msg.direction === 'sent' ? 'SENT' : 'RECV'}
                    </span>
                    <pre class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 text-xs font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap break-all">{msg.content}</pre>
                    <span class="shrink-0 text-[10px] text-gray-400 font-mono">
                      {new Date(msg.time).toLocaleTimeString()}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Send Message Bar -->
          <div class="px-5 py-3 border-t border-gray-100 dark:border-gray-700 shrink-0">
            <div class="flex items-center gap-2">
              <textarea
                bind:value={wsMessage}
                class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400 resize-none"
                placeholder={$_('apiTester__ws_message_placeholder') || 'Type message...'}
                rows={1}
                onkeydown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendWsMessage(); } }}
              ></textarea>
              <button
                onclick={sendWsMessage}
                disabled={wsState !== 'connected' || !wsMessage.trim()}
                class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-bold text-white bg-orange-600 hover:bg-orange-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
              >
                <Send size={14} />
              </button>
              <button
                onclick={clearWsMessages}
                class="p-2.5 text-gray-400 hover:text-red-500 dark:hover:text-red-400 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-700 transition-all"
                title={$_('apiTester__ws_clear') || 'Clear messages'}
              >
                <Trash2 size={16} />
              </button>
            </div>
          </div>
        </div>
      {/if}
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
