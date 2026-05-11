<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { _ } from "svelte-i18n";
  import { api } from "$lib/apis";
  import { requestConfirm } from "$lib/stores/confirmStore";
  import type { AiConfig } from "$lib/types";
  import {
    AlertCircle,
    Sparkles,
    LoaderCircle,
    Eye,
    EyeOff,
    Settings2,
    X,
  } from "lucide-svelte";

  let embeddings_url = "";
  let embeddings_api_key = "";
  let embeddings_model = "";
  let llm_url = "";
  let llm_api_key = "";
  let llm_model = "";
  let pageError = "";
  let pageSuccess = "";
  let modalError = "";
  let modalSuccess = "";
  let savingConfig = false;
  let resettingConfig = false;
  let configModalOpen = false;
  let draftEmbeddingsUrl = "";
  let draftEmbeddingsApiKey = "";
  let draftEmbeddingsModel = "";
  let draftLlmUrl = "";
  let draftLlmApiKey = "";
  let draftLlmModel = "";
  let showApiKey = false;
  let showLlmApiKey = false;

  function openConfigModal() {
    modalError = "";
    modalSuccess = "";
    draftEmbeddingsUrl = embeddings_url;
    draftEmbeddingsApiKey = embeddings_api_key;
    draftEmbeddingsModel = embeddings_model;
    draftLlmUrl = llm_url;
    draftLlmApiKey = llm_api_key;
    draftLlmModel = llm_model;
    showApiKey = false;
    showLlmApiKey = false;
    configModalOpen = true;
  }

  function closeConfigModal() {
    if (savingConfig) return;
    modalError = "";
    modalSuccess = "";
    showApiKey = false;
    showLlmApiKey = false;
    configModalOpen = false;
  }

  async function loadAiConfig() {
    try {
      const response = await api.admin.aiConfig();
      const data = await response.json();

      if (!response.ok) {
        modalError = "Failed to load AI config";
        return;
      }

      const config: AiConfig = data.config || {};
      embeddings_url = config.embeddings_url || "";
      embeddings_api_key = config.embeddings_api_key || "";
      embeddings_model = config.embeddings_model || "";
      llm_url = config.llm_url || "";
      llm_api_key = config.llm_api_key || "";
      llm_model = config.llm_model || "";
      draftEmbeddingsUrl = embeddings_url;
      draftEmbeddingsApiKey = embeddings_api_key;
      draftEmbeddingsModel = embeddings_model;
      draftLlmUrl = llm_url;
      draftLlmApiKey = llm_api_key;
      draftLlmModel = llm_model;
    } catch (error) {
      console.error("Failed to load AI config:", error);
      modalError = "Failed to load AI config";
    }
  }

  async function saveAiConfig() {
    const confirmed = await requestConfirm({
      title: "Save AI Configuration",
      message: "Apply this AI configuration now? The AI chat feature will use these settings.",
      confirmText: "Save",
      cancelText: "Cancel",
      type: "warning",
    });

    if (!confirmed) return;

    savingConfig = true;
    modalError = "";
    modalSuccess = "";

    try {
      const response = await api.admin.updateAiConfig({
        embeddings_url: draftEmbeddingsUrl || null,
        embeddings_api_key: draftEmbeddingsApiKey || null,
        embeddings_model: draftEmbeddingsModel || null,
        llm_url: draftLlmUrl || null,
        llm_api_key: draftLlmApiKey || null,
        llm_model: draftLlmModel || null,
      });
      const data = await response.json();

      if (!response.ok) {
        modalError = data?.error || "Failed to save AI config";
        return;
      }

      pageSuccess = "AI configuration saved successfully";
      modalSuccess = "AI configuration saved successfully";
      configModalOpen = false;
      await loadAiConfig();
    } catch (error) {
      console.error("Failed to save AI config:", error);
      modalError = "Failed to save AI config";
    } finally {
      savingConfig = false;
    }
  }

  async function resetAiConfig() {
    const confirmed = await requestConfirm({
      title: "Reset AI Configuration",
      message: "Reset AI configuration to environment defaults? This will remove all custom settings.",
      confirmText: "Reset",
      cancelText: "Cancel",
      type: "danger",
    });

    if (!confirmed) return;

    resettingConfig = true;
    modalError = "";
    modalSuccess = "";

    try {
      const response = await api.admin.resetAiConfig();
      const data = await response.json();

      if (!response.ok) {
        modalError = "Failed to reset AI config";
        return;
      }

      pageSuccess = "AI configuration reset successfully";
      modalSuccess = "AI configuration reset successfully";
      configModalOpen = false;
      await loadAiConfig();
    } catch (error) {
      console.error("Failed to reset AI config:", error);
      modalError = "Failed to reset AI config";
    } finally {
      resettingConfig = false;
    }
  }

  onMount(() => {
    if (!browser) return;
    void loadAiConfig();
  });
</script>

<svelte:head>
  <title>AI Settings</title>
  <meta name="description" content="Configure AI embeddings and chat settings" />
</svelte:head>

<div class="space-y-6 animate-fade-in">
  <section class="overflow-hidden rounded-[30px] border border-slate-200/80 bg-white text-slate-950 shadow-[0_20px_60px_rgba(15,23,42,0.08)] dark:border-slate-800/80 dark:bg-slate-950 dark:text-white dark:shadow-[0_30px_80px_rgba(15,23,42,0.45)]">
    <div class="bg-[radial-gradient(circle_at_top_left,_rgba(139,92,246,0.14),_transparent_35%),linear-gradient(135deg,_#faf5ff_0%,_#f3e8ff_48%,_#ede9fe_100%)] px-6 py-7 dark:bg-[radial-gradient(circle_at_top_left,_rgba(139,92,246,0.24),_transparent_35%),linear-gradient(135deg,_#020617_0%,_#0f172a_48%,_#1e1b4b_100%)] sm:px-8">
      <div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
        <div class="min-w-0">
          <div class="mb-4 inline-flex rounded-2xl bg-violet-500/10 p-3 text-violet-700 ring-1 ring-violet-500/15 backdrop-blur dark:bg-white/10 dark:text-white dark:ring-white/15">
            <Sparkles size={22} />
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <h2 class="text-2xl font-bold tracking-tight sm:text-3xl">
              AI Settings
            </h2>
            <span class="rounded-full border border-violet-500/20 bg-violet-500/10 px-3 py-1 text-xs font-semibold text-violet-700 dark:border-violet-400/20 dark:bg-violet-400/10 dark:text-violet-100">
              Admin Only
            </span>
          </div>
          <p class="mt-3 max-w-2xl text-sm leading-6 text-slate-600 dark:text-slate-300/85">
            Configure AI embeddings endpoint for semantic task search and chat features
          </p>
        </div>
      </div>
    </div>
  </section>

  <section class="rounded-[28px] border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
    <div class="flex items-start justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="rounded-2xl bg-violet-500/10 p-3 text-violet-500">
          <Sparkles size={18} />
        </div>
        <div>
          <h3 class="text-lg font-bold text-gray-900 dark:text-white">
            Embeddings Configuration
          </h3>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Configure the AI embeddings API endpoint and credentials
          </p>
        </div>
      </div>
      <button
        type="button"
        class="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-gray-200 bg-white text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-200 dark:hover:bg-gray-800"
        on:click={openConfigModal}
        aria-label="Open AI settings"
        title="Open AI settings"
      >
        <Settings2 size={18} />
      </button>
    </div>

    <div class="mt-5 space-y-6">
      <div>
        <h4 class="mb-3 text-sm font-semibold text-gray-700 dark:text-gray-300">Embeddings (Search)</h4>
        <div class="grid gap-3 lg:grid-cols-2">
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              Embeddings URL
            </p>
            <p class="mt-2 truncate text-sm text-gray-900 dark:text-white">
              {embeddings_url || "Not configured"}
            </p>
          </div>
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              Model
            </p>
            <p class="mt-2 text-sm text-gray-900 dark:text-white">
              {embeddings_model || "Not configured"}
            </p>
          </div>
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60 lg:col-span-2">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              API Key
            </p>
            <p class="mt-2 font-mono text-sm text-gray-900 dark:text-white">
              {embeddings_api_key ? "••••••••••••••••" : "Not configured"}
            </p>
          </div>
        </div>
      </div>

      <div>
        <h4 class="mb-3 text-sm font-semibold text-gray-700 dark:text-gray-300">LLM (Task Generation)</h4>
        <div class="grid gap-3 lg:grid-cols-2">
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              LLM URL
            </p>
            <p class="mt-2 truncate text-sm text-gray-900 dark:text-white">
              {llm_url || "Not configured"}
            </p>
          </div>
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              Model
            </p>
            <p class="mt-2 text-sm text-gray-900 dark:text-white">
              {llm_model || "Not configured"}
            </p>
          </div>
          <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60 lg:col-span-2">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
              API Key
            </p>
            <p class="mt-2 font-mono text-sm text-gray-900 dark:text-white">
              {llm_api_key ? "••••••••••••••••" : "Not configured"}
            </p>
          </div>
        </div>
      </div>
    </div>

    {#if pageError}
      <div class="mt-4 rounded-2xl border border-red-200 bg-red-50/70 px-4 py-3 text-sm text-red-700 dark:border-red-500/20 dark:bg-red-500/10 dark:text-red-200">
        <div class="flex items-start gap-3">
          <AlertCircle size={18} class="mt-0.5 shrink-0" />
          <p>{pageError}</p>
        </div>
      </div>
    {/if}

    {#if pageSuccess}
      <div class="mt-4 rounded-2xl border border-emerald-200 bg-emerald-50/80 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-500/20 dark:bg-emerald-500/10 dark:text-emerald-200">
        {pageSuccess}
      </div>
    {/if}
  </section>
</div>

{#if configModalOpen}
  <div class="fixed inset-0 z-[1150] flex items-center justify-center p-4">
    <button
      type="button"
      class="absolute inset-0 bg-slate-950/55 backdrop-blur-sm dark:bg-slate-950/75"
      on:click={closeConfigModal}
      aria-label="Close settings"
    ></button>

    <div class="relative z-[1] w-full max-w-3xl overflow-hidden rounded-[28px] border border-slate-200/80 bg-white text-slate-900 shadow-[0_24px_80px_rgba(15,23,42,0.14)] dark:border-slate-700/80 dark:bg-slate-900 dark:text-white dark:shadow-[0_30px_100px_rgba(15,23,42,0.6)]">
      <div class="bg-[radial-gradient(circle_at_top_left,_rgba(139,92,246,0.12),_transparent_35%),linear-gradient(135deg,_#faf5ff_0%,_#f3e8ff_55%,_#ede9fe_100%)] px-6 py-5 dark:bg-[radial-gradient(circle_at_top_left,_rgba(139,92,246,0.18),_transparent_35%),linear-gradient(135deg,_#0f172a_0%,_#111827_55%,_#1e1b4b_100%)] sm:px-7">
        <div class="flex items-start justify-between gap-4">
          <div class="flex items-center gap-3">
            <div class="rounded-2xl bg-violet-500/10 p-3 text-violet-600 ring-1 ring-violet-500/20 dark:text-violet-300 dark:ring-violet-400/20">
              <Settings2 size={18} />
            </div>
            <div>
              <h3 class="text-lg font-bold">AI Configuration</h3>
              <p class="text-sm text-slate-600 dark:text-slate-300">
                Configure embeddings API endpoint and credentials
              </p>
            </div>
          </div>
          <button
            type="button"
            class="inline-flex h-10 w-10 min-h-10 min-w-10 shrink-0 items-center justify-center rounded-full border border-slate-300 bg-white/70 text-slate-600 shadow-[inset_0_1px_0_rgba(255,255,255,0.2)] transition hover:border-slate-400 hover:bg-white hover:text-slate-900 dark:border-slate-600/70 dark:bg-slate-950/55 dark:text-slate-300 dark:shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] dark:hover:border-slate-500 dark:hover:bg-slate-900 dark:hover:text-white"
            on:click={closeConfigModal}
            aria-label="Close settings"
          >
            <X size={16} strokeWidth={2.25} />
          </button>
        </div>
      </div>

      <div class="px-6 py-6 sm:px-7">
        {#if modalError}
          <div class="mb-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-500/20 dark:bg-red-500/10 dark:text-red-100">
            {modalError}
          </div>
        {/if}

        {#if modalSuccess}
          <div class="mb-4 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-500/20 dark:bg-emerald-500/10 dark:text-emerald-100">
            {modalSuccess}
          </div>
        {/if}

        <div class="grid gap-4">
          <div>
            <h5 class="mb-3 text-sm font-semibold text-slate-700 dark:text-slate-200">Embeddings (Search)</h5>
            <div class="space-y-4">
              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>Embeddings URL</span>
                <input bind:value={draftEmbeddingsUrl} placeholder="http://example.com/v1/embeddings" class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white" />
              </label>

              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>API Key</span>
                <div class="relative">
                  <input
                    bind:value={draftEmbeddingsApiKey}
                    type={showApiKey ? "text" : "password"}
                    placeholder="sk-..."
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 pr-12 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white"
                  />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 inline-flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full text-slate-400 transition hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-slate-800 dark:hover:text-white"
                    aria-label={showApiKey ? "Hide API key" : "Show API key"}
                    on:click={() => (showApiKey = !showApiKey)}
                  >
                    {#if showApiKey}
                      <EyeOff size={16} />
                    {:else}
                      <Eye size={16} />
                    {/if}
                  </button>
                </div>
              </label>

              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>Model</span>
                <input bind:value={draftEmbeddingsModel} placeholder="mistral/mistral-embed" class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white" />
              </label>
            </div>
          </div>

          <div class="border-t border-slate-200 pt-4 dark:border-slate-700">
            <h5 class="mb-3 text-sm font-semibold text-slate-700 dark:text-slate-200">LLM (Task Generation)</h5>
            <div class="space-y-4">
              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>LLM URL</span>
                <input bind:value={draftLlmUrl} placeholder="http://example.com/v1/chat/completions" class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white" />
              </label>

              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>API Key</span>
                <div class="relative">
                  <input
                    bind:value={draftLlmApiKey}
                    type={showLlmApiKey ? "text" : "password"}
                    placeholder="sk-..."
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 pr-12 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white"
                  />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 inline-flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full text-slate-400 transition hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-slate-800 dark:hover:text-white"
                    aria-label={showLlmApiKey ? "Hide API key" : "Show API key"}
                    on:click={() => (showLlmApiKey = !showLlmApiKey)}
                  >
                    {#if showLlmApiKey}
                      <EyeOff size={16} />
                    {:else}
                      <Eye size={16} />
                    {/if}
                  </button>
                </div>
              </label>

              <label class="flex flex-col gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
                <span>Model</span>
                <input bind:value={draftLlmModel} placeholder="mistral/mistral-7b-instruct" class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none transition focus:border-violet-400 dark:border-slate-700 dark:bg-slate-950/70 dark:text-white" />
              </label>
            </div>
          </div>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <div class="mr-auto">
            <button
              type="button"
              class="rounded-2xl border border-amber-200 bg-amber-50 px-5 py-3 text-sm font-semibold text-amber-700 transition hover:border-amber-300 hover:bg-amber-100 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-600/70 dark:bg-slate-900/70 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:bg-slate-800"
              on:click={resetAiConfig}
              disabled={savingConfig || resettingConfig}
            >
              {#if resettingConfig}
                Reset
              {:else}
                Reset
              {/if}
            </button>
          </div>
          <button
            type="button"
            class="rounded-2xl border border-slate-200 bg-white px-5 py-3 text-sm font-semibold text-slate-700 transition hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-900/70 dark:text-slate-200 dark:hover:bg-slate-800"
            on:click={closeConfigModal}
            disabled={savingConfig || resettingConfig}
          >
            Cancel
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-2xl border border-violet-500/30 bg-[linear-gradient(135deg,_rgba(139,92,246,0.92),_rgba(124,58,237,0.88))] px-5 py-3 text-sm font-semibold text-white shadow-[0_14px_34px_rgba(139,92,246,0.2)] transition hover:border-violet-400/40 hover:bg-[linear-gradient(135deg,_rgba(167,139,250,0.96),_rgba(139,92,246,0.92))] disabled:cursor-not-allowed disabled:border-slate-200 disabled:bg-slate-200 disabled:text-slate-500 disabled:shadow-none dark:disabled:border-slate-700 dark:disabled:bg-slate-700 dark:disabled:text-slate-400"
            disabled={savingConfig || resettingConfig}
            on:click={saveAiConfig}
          >
            {#if savingConfig}
              <LoaderCircle size={16} class="animate-spin" />
            {/if}
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

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
