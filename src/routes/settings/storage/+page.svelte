<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { _ } from "svelte-i18n";
  import { API_BASE_URL, api } from "$lib/apis";
  import { requestConfirm } from "$lib/stores/confirmStore";
  import type { StorageObjectItem } from "$lib/types";
  import {
    AlertCircle,
    DatabaseBackup,
    HardDrive,
    LoaderCircle,
    Eye,
    Trash2,
    X,
  } from "lucide-svelte";

  let bucketName = "khunphaen-assets";
  let storageEndpoint = "RustFS / S3-compatible";
  let usedGb = 0;
  let objectCount = 0;
  let storageError = "";
  let objects: StorageObjectItem[] = [];
  let loadingObjects = true;
  let deletingKey = "";
  let currentPage = 1;
  let pageSize = 10;
  let totalObjects = 0;
  let totalPages = 1;
  let previewKey = "";
  let previewMimeType = "";

  function formatStorageSize(sizeGb: number) {
    const sizeBytes = Math.max(0, sizeGb) * 1024 * 1024 * 1024;
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = sizeBytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    const fractionDigits = value >= 100 ? 0 : value >= 10 ? 1 : 2;
    return `${value.toFixed(fractionDigits)} ${units[unitIndex]}`;
  }

  function formatBytes(sizeBytes: number) {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = Math.max(0, sizeBytes);
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    const fractionDigits = value >= 100 ? 0 : value >= 10 ? 1 : 2;
    return `${value.toFixed(fractionDigits)} ${units[unitIndex]}`;
  }

  function formatTimestamp(value?: string | null) {
    if (!value) return "-";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
  }

  function buildFileUrl(key: string) {
    return `${API_BASE_URL}/files/${encodeURIComponent(key)}`;
  }

  function openPreview(item: StorageObjectItem) {
    previewKey = item.key;
    previewMimeType = item.mime_type || "";
  }

  function closePreview() {
    previewKey = "";
    previewMimeType = "";
  }

  function isImageFile(mimeType: string, key: string) {
    if (mimeType.startsWith("image/")) return true;
    const lowerKey = key.toLowerCase();
    return [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"].some(
      (ext) => lowerKey.endsWith(ext),
    );
  }

  async function loadStorageStats() {
    storageError = "";

    try {
      const response = await api.admin.storageStats();
      const data = await response.json();

      if (!response.ok) {
        storageError = data.error || "Failed to load storage stats";
        return;
      }

      bucketName = data.bucket || bucketName;
      storageEndpoint = data.endpoint || storageEndpoint;
      usedGb = Number(data.used_bytes || 0) / 1024 / 1024 / 1024;
      objectCount = Number(data.object_count || 0);
    } catch (error) {
      console.error("Failed to load storage stats:", error);
      storageError = "Failed to load storage stats";
    }
  }

  async function loadStorageObjects() {
    loadingObjects = true;
    storageError = "";

    try {
      const response = await api.admin.listStorageObjects(currentPage, pageSize);
      const data = await response.json();

      if (!response.ok) {
        storageError = data.error || "Failed to load storage objects";
        return;
      }

      objects = data.objects || [];
      totalObjects = Number(data.total || objects.length || 0);
      totalPages = Math.max(
        1,
        Number(data.pages || Math.ceil(Math.max(totalObjects, 1) / pageSize)),
      );
      currentPage = Math.max(1, Number(data.page || currentPage));
    } catch (error) {
      console.error("Failed to load storage objects:", error);
      storageError = "Failed to load storage objects";
    } finally {
      loadingObjects = false;
    }
  }

  async function handleDeleteObject(item: StorageObjectItem) {
    const confirmed = await requestConfirm({
      title: $_("settings__storage_delete_title"),
      message: $_("settings__storage_delete_message", {
        values: { key: item.key },
      }),
      confirmText: $_("settings__storage_delete_confirm"),
      cancelText: $_("common.cancel"),
      type: "danger",
    });

    if (!confirmed) return;

    deletingKey = item.key;
    storageError = "";

    try {
      const response = await api.admin.deleteStorageObject(
        encodeURIComponent(item.key),
      );
      const data = await response.json();

      if (!response.ok) {
        storageError = data.error || "Failed to delete storage object";
        return;
      }

      await Promise.all([loadStorageStats(), loadStorageObjects()]);
    } catch (error) {
      console.error("Failed to delete storage object:", error);
      storageError = "Failed to delete storage object";
    } finally {
      deletingKey = "";
    }
  }

  async function handlePageSizeChange(event: Event) {
    const next = Number((event.target as HTMLSelectElement).value);
    if (!Number.isFinite(next) || next <= 0) return;
    pageSize = next;
    currentPage = 1;
    await loadStorageObjects();
  }

  onMount(() => {
    if (!browser) return;

    void loadStorageStats();
    void loadStorageObjects();
  });

  $: normalizedUsedGb =
    Number.isFinite(usedGb) && usedGb >= 0 ? Number(usedGb) : 0;
  $: if (currentPage > totalPages) currentPage = totalPages;
  $: pageStart = totalObjects === 0 ? 0 : (currentPage - 1) * pageSize + 1;
  $: pageEnd = Math.min(currentPage * pageSize, totalObjects);

  async function goToPage(page: number) {
    if (page < 1 || page > totalPages) return;
    currentPage = page;
    await loadStorageObjects();
  }
</script>

<div class="space-y-6 animate-fade-in">
  <section class="overflow-hidden rounded-[30px] border border-slate-800/80 bg-slate-950 text-white shadow-[0_30px_80px_rgba(15,23,42,0.45)]">
    <div class="bg-[radial-gradient(circle_at_top_left,_rgba(59,130,246,0.24),_transparent_35%),linear-gradient(135deg,_#020617_0%,_#0f172a_48%,_#172554_100%)] px-6 py-7 sm:px-8">
      <div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
        <div class="min-w-0">
          <div class="mb-4 inline-flex rounded-2xl bg-white/10 p-3 ring-1 ring-white/15 backdrop-blur">
            <HardDrive size={22} />
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <h2 class="text-2xl font-bold tracking-tight sm:text-3xl">
              {$_("settings__storage_title")}
            </h2>
            <span class="rounded-full border border-sky-400/20 bg-sky-400/10 px-3 py-1 text-xs font-semibold text-sky-100">
              {$_("settings__storage_badge")}
            </span>
          </div>
          <p class="mt-3 max-w-2xl text-sm leading-6 text-slate-300/85">
            {$_("settings__storage_subtitle")}
          </p>
        </div>

        <div class="w-full max-w-sm rounded-[28px] border border-white/10 bg-white/5 p-5 backdrop-blur">
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.22em] text-slate-400">
                {$_("settings__storage_capacity_used")}
              </p>
              <p class="mt-2 text-3xl font-black tracking-tight text-white">
                {formatStorageSize(normalizedUsedGb)}
              </p>
              <p class="mt-1 text-sm text-slate-300">
                {objectCount.toLocaleString()} {$_("settings__storage_objects_suffix")}
              </p>
            </div>
            <div class="rounded-2xl bg-sky-500/10 p-3 text-sky-300">
              <DatabaseBackup size={18} />
            </div>
          </div>

          <div class="mt-4 flex items-center justify-between text-xs text-slate-400">
            <span>{$_("settings__storage_capacity_live")}</span>
            <span>{bucketName}</span>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="rounded-[28px] border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
    <div class="flex items-center gap-3">
      <div class="rounded-2xl bg-indigo-500/10 p-3 text-indigo-500">
        <HardDrive size={18} />
      </div>
      <div>
        <h3 class="text-lg font-bold text-gray-900 dark:text-white">
          {$_("settings__storage_bucket_title")}
        </h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {$_("settings__storage_bucket_subtitle")}
        </p>
      </div>
    </div>

    <div class="mt-5 grid gap-3 lg:grid-cols-2">
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_field_bucket")}
        </p>
        <p class="mt-2 font-mono text-sm text-gray-900 dark:text-white">
          {bucketName}
        </p>
      </div>
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_field_endpoint")}
        </p>
        <p class="mt-2 text-sm text-gray-900 dark:text-white">
          {storageEndpoint}
        </p>
      </div>
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_metric_used")}
        </p>
        <p class="mt-2 text-sm font-semibold text-gray-900 dark:text-white">
          {formatStorageSize(normalizedUsedGb)}
        </p>
      </div>
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_metric_objects")}
        </p>
        <p class="mt-2 text-sm font-semibold text-gray-900 dark:text-white">
          {objectCount.toLocaleString()}
        </p>
      </div>
    </div>

    {#if storageError}
      <div class="mt-4 rounded-2xl border border-red-200 bg-red-50/70 px-4 py-3 text-sm text-red-700 dark:border-red-500/20 dark:bg-red-500/10 dark:text-red-200">
        <div class="flex items-start gap-3">
          <AlertCircle size={18} class="mt-0.5 shrink-0" />
          <p>{storageError}</p>
        </div>
      </div>
    {/if}
  </section>

  <section class="rounded-[28px] border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h3 class="text-lg font-bold text-gray-900 dark:text-white">
          {$_("settings__storage_files_title")}
        </h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {$_("settings__storage_files_subtitle")}
        </p>
      </div>
      <div class="flex items-center gap-3">
        <label class="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          <span>{$_("settings__storage_pagination_limit")}</span>
          <select
            class="rounded-xl border border-gray-200 bg-white px-3 py-2 text-sm font-semibold normal-case tracking-normal text-gray-700 outline-none transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-200 dark:hover:bg-gray-800"
            bind:value={pageSize}
            on:change={handlePageSizeChange}
          >
            <option value={10}>10</option>
            <option value={20}>20</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
          </select>
        </label>
        <div class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {totalObjects.toLocaleString()} {$_("settings__storage_objects_suffix")}
        </div>
      </div>
    </div>

    <div class="mt-5 overflow-hidden rounded-2xl border border-gray-200 dark:border-gray-700">
      <div class="overflow-x-auto">
        <table class="w-full text-left">
          <thead class="bg-gray-50 dark:bg-gray-900/60">
            <tr>
              <th class="px-4 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
                {$_("settings__storage_table_key")}
              </th>
              <th class="px-4 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
                {$_("settings__storage_table_size")}
              </th>
              <th class="px-4 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
                {$_("settings__storage_table_modified")}
              </th>
              <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
                {$_("settings__storage_table_action")}
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            {#if loadingObjects}
              <tr>
                <td colspan="4" class="px-4 py-10 text-center text-sm text-gray-500 dark:text-gray-400">
                  {$_("settings__storage_loading")}
                </td>
              </tr>
            {:else if objects.length === 0}
              <tr>
                <td colspan="4" class="px-4 py-10 text-center text-sm text-gray-500 dark:text-gray-400">
                  {$_("settings__storage_empty")}
                </td>
              </tr>
            {:else}
              {#each objects as item}
                <tr class="bg-white dark:bg-gray-800">
                  <td class="px-4 py-3 align-top">
                    <div class="max-w-[520px] break-all text-sm text-gray-900 dark:text-white">
                      {item.key}
                    </div>
                  </td>
                  <td class="px-4 py-3 align-top text-sm text-gray-600 dark:text-gray-300">
                    {formatBytes(item.size_bytes)}
                  </td>
                  <td class="px-4 py-3 align-top text-sm text-gray-600 dark:text-gray-300">
                    {formatTimestamp(item.last_modified)}
                  </td>
                  <td class="px-4 py-3 text-right align-top">
                    <div class="inline-flex items-center gap-2">
                      <button
                        type="button"
                        class="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-gray-200 text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-900/60"
                        on:click={() => openPreview(item)}
                        title={$_("settings__storage_view_action")}
                      >
                        <Eye size={15} />
                      </button>
                      <button
                        type="button"
                        class="inline-flex items-center gap-2 rounded-xl border border-red-200 px-3 py-2 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50 dark:border-red-500/20 dark:text-red-400 dark:hover:bg-red-500/10"
                        disabled={deletingKey === item.key}
                        on:click={() => handleDeleteObject(item)}
                      >
                        {#if deletingKey === item.key}
                          <LoaderCircle size={15} class="animate-spin" />
                        {:else}
                          <Trash2 size={15} />
                        {/if}
                        {$_("settings__storage_delete_action")}
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>

    {#if !loadingObjects && totalObjects > 0}
      <div class="mt-4 flex flex-col gap-3 border-t border-gray-200 pt-4 dark:border-gray-700 sm:flex-row sm:items-center sm:justify-between">
        <div class="text-sm text-gray-500 dark:text-gray-400">
          {$_("settings__storage_pagination_summary", {
            values: {
              start: pageStart,
              end: pageEnd,
              total: totalObjects,
            },
          })}
        </div>

        <div class="flex items-center gap-2 self-end sm:self-auto">
          <button
            type="button"
            class="rounded-xl border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-600 transition-colors hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-900/60"
            on:click={async () => await goToPage(currentPage - 1)}
            disabled={currentPage === 1}
          >
            {$_("settings__storage_pagination_prev")}
          </button>

          <div class="rounded-xl border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-700 dark:border-gray-700 dark:text-gray-200">
            {currentPage} / {totalPages}
          </div>

          <button
            type="button"
            class="rounded-xl border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-600 transition-colors hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-900/60"
            on:click={async () => await goToPage(currentPage + 1)}
            disabled={currentPage === totalPages}
          >
            {$_("settings__storage_pagination_next")}
          </button>
        </div>
      </div>
    {/if}
  </section>
</div>

{#if previewKey}
  <div class="fixed inset-0 z-[1200] flex items-center justify-center p-4">
    <button
      type="button"
      class="absolute inset-0 bg-black/70 backdrop-blur-sm"
      on:click={closePreview}
      aria-label={$_("settings__storage_preview_close")}
    ></button>

    <div class="relative flex h-[85vh] w-full max-w-6xl flex-col overflow-hidden rounded-[28px] border border-gray-200 bg-white shadow-2xl dark:border-gray-700 dark:bg-gray-900">
      <div class="flex items-center justify-between gap-4 border-b border-gray-200 px-5 py-4 dark:border-gray-700">
        <div class="min-w-0">
          <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
            {$_("settings__storage_preview_title")}
          </p>
          <p class="mt-1 truncate text-sm font-semibold text-gray-900 dark:text-white">
            {previewKey}
          </p>
        </div>

        <div class="flex items-center gap-2">
          <a
            href={buildFileUrl(previewKey)}
            target="_blank"
            rel="noopener noreferrer"
            class="rounded-xl border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
          >
            {$_("settings__storage_preview_open_new")}
          </a>
          <button
            type="button"
            class="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-gray-200 text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
            on:click={closePreview}
            aria-label={$_("settings__storage_preview_close")}
          >
            <X size={18} />
          </button>
        </div>
      </div>

      <div class="min-h-0 flex-1 bg-gray-100 dark:bg-gray-950">
        {#if isImageFile(previewMimeType, previewKey)}
          <div class="flex h-full w-full items-center justify-center overflow-auto bg-[radial-gradient(circle_at_center,_rgba(148,163,184,0.18),_transparent_55%)] p-4">
            <img
              src={buildFileUrl(previewKey)}
              alt={previewKey}
              class="max-h-full max-w-full object-contain"
            />
          </div>
        {:else}
          <iframe
            src={buildFileUrl(previewKey)}
            title={previewKey}
            class="h-full w-full border-0 bg-white dark:bg-gray-950"
          ></iframe>
        {/if}
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
