<script lang="ts">
  import { onMount } from "svelte";
  import { ExternalLink, Figma, Link as LinkIcon, X } from "lucide-svelte";

  export let url: string;
  export let readonly = false;
  export let onRemove: (() => void) | null = null;

  type OEmbedData = {
    title?: string;
    thumbnail_url?: string;
    provider_name?: string;
  };

  let oEmbed: OEmbedData | null = null;
  let loading = true;

  const FIGMA_PATTERN = /figma\.com\/(proto|design|file|board)\//i;
  $: isFigma = FIGMA_PATTERN.test(url);

  $: displayUrl = (() => {
    try {
      const u = new URL(url);
      return u.hostname + u.pathname.slice(0, 40) + (u.pathname.length > 40 ? "…" : "");
    } catch {
      return url.slice(0, 50);
    }
  })();

  onMount(async () => {
    if (!isFigma) { loading = false; return; }
    try {
      const res = await fetch(
        `https://www.figma.com/api/oembed?url=${encodeURIComponent(url)}`,
      );
      if (res.ok) oEmbed = await res.json();
    } catch {
      // fallback to styled card
    } finally {
      loading = false;
    }
  });
</script>

<div class="group relative flex items-start gap-3 p-3 rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 hover:border-gray-300 dark:hover:border-gray-600 transition-colors">

  {#if isFigma && oEmbed?.thumbnail_url && !loading}
    <!-- Figma with thumbnail -->
    <a href={url} target="_blank" rel="noopener noreferrer" class="flex items-start gap-3 flex-1 min-w-0">
      <img
        src={oEmbed.thumbnail_url}
        alt={oEmbed.title || "Figma preview"}
        class="w-20 h-14 object-cover rounded-lg border border-gray-200 dark:border-gray-700 shrink-0 bg-gray-200 dark:bg-gray-700"
      />
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5 mb-0.5">
          <Figma size={12} class="text-[#F24E1E] shrink-0" />
          <span class="text-[11px] text-[#F24E1E] font-medium">Figma</span>
        </div>
        <p class="text-sm font-medium text-gray-800 dark:text-gray-200 truncate leading-snug">
          {oEmbed.title || "Figma file"}
        </p>
        <p class="text-[11px] text-gray-400 truncate mt-0.5">{displayUrl}</p>
      </div>
      <ExternalLink size={14} class="text-gray-400 shrink-0 mt-0.5" />
    </a>

  {:else if isFigma}
    <!-- Figma without thumbnail (loading or failed) -->
    <a href={url} target="_blank" rel="noopener noreferrer" class="flex items-center gap-3 flex-1 min-w-0">
      <div class="w-9 h-9 rounded-lg bg-[#F24E1E]/10 flex items-center justify-center shrink-0">
        <Figma size={18} class="text-[#F24E1E]" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
          {loading ? "กำลังโหลด..." : (oEmbed?.title || "Figma")}
        </p>
        <p class="text-[11px] text-gray-400 truncate">{displayUrl}</p>
      </div>
      <ExternalLink size={14} class="text-gray-400 shrink-0" />
    </a>

  {:else}
    <!-- Generic link -->
    <a href={url} target="_blank" rel="noopener noreferrer" class="flex items-center gap-3 flex-1 min-w-0">
      <div class="w-9 h-9 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center shrink-0">
        <LinkIcon size={16} class="text-gray-500 dark:text-gray-400" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm text-gray-700 dark:text-gray-300 truncate">{displayUrl}</p>
      </div>
      <ExternalLink size={14} class="text-gray-400 shrink-0" />
    </a>
  {/if}

  {#if !readonly && onRemove}
    <button
      type="button"
      on:click|stopPropagation={onRemove}
      class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 w-5 h-5 rounded-full bg-gray-200 dark:bg-gray-700 hover:bg-red-100 dark:hover:bg-red-900/30 hover:text-red-500 flex items-center justify-center transition-all"
    >
      <X size={11} />
    </button>
  {/if}
</div>
