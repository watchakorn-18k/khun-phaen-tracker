<script lang="ts">
  import { page } from "$app/stores";
  import { base } from "$app/paths";
  import { ArrowLeft, Home } from "lucide-svelte";
  import { _ } from "svelte-i18n";
</script>

<svelte:head>
  <title>{$page.status} - {$page.error?.message || $_("page__error")}</title>
</svelte:head>

<div
  class="min-h-[75vh] flex flex-col items-center justify-center px-4 sm:px-6 lg:px-8 animate-fade-in relative overflow-hidden"
>
  <div
    class="relative z-10 text-center max-w-2xl mx-auto flex flex-col items-center"
  >
    <!-- Error Status & Message -->
    <h1
      class="text-6xl sm:text-8xl font-black dark:text-white tracking-tighter mb-4 drop-shadow-sm bg-clip-text text-transparent bg-linear-to-b from-gray-900 to-gray-600 dark:from-white dark:to-gray-400"
    >
      {$page.status}
    </h1>

    <h2
      class="text-2xl sm:text-3xl font-bold text-gray-800 dark:text-gray-200 mb-6 tracking-tight"
    >
      {$page.error?.message || $_("error__default_title")}
    </h2>

    <p
      class="text-gray-500 dark:text-gray-400 text-lg mb-10 max-w-md mx-auto leading-relaxed"
    >
      {#if $page.status === 404}
        {$_("error__404_message")}
      {:else}
        {$_("error__default_message")}
      {/if}
    </p>

    <!-- Actions -->
    <div class="flex flex-col sm:flex-row items-center gap-4 w-full sm:w-auto">
      <button
        on:click={() => window.history.back()}
        class="w-full sm:w-auto px-6 py-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-xl font-semibold transition-all flex items-center justify-center gap-2 shadow-sm ring-1 ring-black/5 dark:ring-white/5 active:scale-95"
      >
        <ArrowLeft size={18} />
        {$_("error__btn_back")}
      </button>

      <a
        href="{base}/"
        class="w-full sm:w-auto px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl font-semibold transition-all shadow-lg shadow-indigo-600/20 ring-1 ring-indigo-500/50 flex items-center justify-center gap-2 active:scale-95"
      >
        <Home size={18} />
        {$_("error__btn_home")}
      </a>
    </div>
  </div>
</div>
