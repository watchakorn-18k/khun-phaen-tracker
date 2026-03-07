<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { _ } from "svelte-i18n";
  import { API_BASE_URL, api } from "$lib/apis";
  import { requestConfirm } from "$lib/stores/confirmStore";
  import type { StorageConfig, StorageObjectItem, StorageProvider } from "$lib/types";
  import {
    AlertCircle,
    DatabaseBackup,
    HardDrive,
    LoaderCircle,
    Eye,
    EyeOff,
    Settings2,
    Trash2,
    X,
  } from "lucide-svelte";

  let bucketName = "khunphaen-assets";
  let storageEndpoint = "RustFS / S3-compatible";
  let usedGb = 0;
  let objectCount = 0;
  let pageError = "";
  let pageSuccess = "";
  let modalError = "";
  let modalSuccess = "";
  let objects: StorageObjectItem[] = [];
  let loadingObjects = true;
  let savingConfig = false;
  let resettingConfig = false;
  let deletingKey = "";
  let currentPage = 1;
  let pageSize = 10;
  let totalObjects = 0;
  let totalPages = 1;
  let previewKey = "";
  let providerModalOpen = false;
  let storageProvider: StorageProvider = "env";
  let storageRegion = "us-east-1";
  let storageAccessKey = "";
  let storageSecretKey = "";
  let draftStorageProvider: StorageProvider = "env";
  let draftBucketName = "";
  let draftStorageRegion = "us-east-1";
  let draftStorageEndpoint = "";
  let draftStorageAccessKey = "";
  let draftStorageSecretKey = "";
  let showAccessKey = false;
  let showSecretKey = false;
  const rustfsDocsUrl = "https://docs.rustfs.com/en/introduction.html";
  const rustfsSdkUrl = "https://docs.rustfs.com/developer/sdk/rust.html";
  const awsS3DocsUrl =
    "https://docs.aws.amazon.com/AmazonS3/latest/userguide/GetStartedWithS3.html";
  const awsS3PolicyDocsUrl =
    "https://docs.aws.amazon.com/AmazonS3/latest/userguide/example-bucket-policies.html";

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

  function mapStorageErrorMessage(message?: string) {
    switch (message) {
      case "Bucket name is required":
        return $_("settings__storage_validation_bucket_required");
      case "Region is required":
        return $_("settings__storage_validation_region_required");
      case "Storage endpoint is required":
        return $_("settings__storage_validation_endpoint_required");
      case "Access key and secret key are required":
        return $_("settings__storage_validation_credentials_required");
      default:
        return null;
    }
  }

  function openPreview(item: StorageObjectItem) {
    previewKey = item.key;
  }

  function closePreview() {
    previewKey = "";
  }

  function openProviderModal() {
    modalError = "";
    modalSuccess = "";
    draftStorageProvider = storageProvider;
    draftBucketName = bucketName;
    draftStorageRegion = storageRegion;
    draftStorageEndpoint = storageEndpoint;
    draftStorageAccessKey = storageAccessKey;
    draftStorageSecretKey = storageSecretKey;
    showAccessKey = false;
    showSecretKey = false;
    providerModalOpen = true;
  }

  function closeProviderModal() {
    if (savingConfig) return;
    modalError = "";
    modalSuccess = "";
    showAccessKey = false;
    showSecretKey = false;
    providerModalOpen = false;
  }

  async function loadStorageStats() {
    pageError = "";

    try {
      const response = await api.admin.storageStats();
      const data = await response.json();

      if (!response.ok) {
        pageError = $_("settings__storage_modal_error_load_stats");
        return;
      }

      bucketName = data.bucket || bucketName;
      storageEndpoint = data.endpoint || storageEndpoint;
      storageRegion = data.region || storageRegion;
      usedGb = Number(data.used_bytes || 0) / 1024 / 1024 / 1024;
      objectCount = Number(data.object_count || 0);
    } catch (error) {
      console.error("Failed to load storage stats:", error);
      pageError = $_("settings__storage_modal_error_load_stats");
    }
  }

  async function loadStorageConfig() {
    try {
      const response = await api.admin.storageConfig();
      const data = await response.json();

      if (!response.ok) {
        modalError = $_("settings__storage_modal_error_load_config");
        return;
      }

      const config: StorageConfig = data.config || {};
      storageProvider = config.provider || "env";
      bucketName = config.bucket || bucketName;
      storageRegion = config.region || storageRegion;
      storageEndpoint = config.endpoint || storageEndpoint;
      storageAccessKey = config.access_key || "";
      storageSecretKey = config.secret_key || "";
      draftStorageProvider = storageProvider;
      draftBucketName = bucketName;
      draftStorageRegion = storageRegion;
      draftStorageEndpoint = storageEndpoint;
      draftStorageAccessKey = storageAccessKey;
      draftStorageSecretKey = storageSecretKey;
    } catch (error) {
      console.error("Failed to load storage config:", error);
      modalError = $_("settings__storage_modal_error_load_config");
    }
  }

  async function saveStorageConfig() {
    const confirmed = await requestConfirm({
      title: $_("settings__storage_modal_confirm_save_title"),
      message: $_("settings__storage_modal_confirm_save_message"),
      confirmText: $_("settings__storage_modal_confirm_save_action"),
      cancelText: $_("common.cancel"),
      type: "warning",
    });

    if (!confirmed) return;

    savingConfig = true;
    modalError = "";
    modalSuccess = "";

    try {
      const response = await api.admin.updateStorageConfig({
        provider: draftStorageProvider,
        bucket: draftBucketName,
        region: draftStorageRegion,
        endpoint: draftStorageProvider === "env" ? draftStorageEndpoint : undefined,
        access_key: draftStorageAccessKey,
        secret_key: draftStorageSecretKey,
      });
      const data = await response.json();

      if (!response.ok) {
        modalError =
          mapStorageErrorMessage(data?.error) ||
          $_("settings__storage_modal_error_save_config");
        return;
      }

      pageSuccess = $_("settings__storage_modal_success_saved");
      modalSuccess = $_("settings__storage_modal_success_saved");
      providerModalOpen = false;
      await Promise.all([loadStorageConfig(), loadStorageStats(), loadStorageObjects()]);
    } catch (error) {
      console.error("Failed to save storage config:", error);
      modalError = $_("settings__storage_modal_error_save_config");
    } finally {
      savingConfig = false;
    }
  }

  async function resetStorageConfig() {
    const confirmed = await requestConfirm({
      title: $_("settings__storage_modal_reset_title"),
      message: $_("settings__storage_modal_reset_message"),
      confirmText: $_("settings__storage_modal_reset"),
      cancelText: $_("common.cancel"),
      type: "danger",
    });

    if (!confirmed) return;

    resettingConfig = true;
    modalError = "";
    modalSuccess = "";

    try {
      const response = await api.admin.resetStorageConfig();
      const data = await response.json();

      if (!response.ok) {
        modalError = $_("settings__storage_modal_error_reset_config");
        return;
      }

      pageSuccess = $_("settings__storage_modal_success_reset");
      modalSuccess = $_("settings__storage_modal_success_reset");
      providerModalOpen = false;
      await Promise.all([loadStorageConfig(), loadStorageStats(), loadStorageObjects()]);
    } catch (error) {
      console.error("Failed to reset storage config:", error);
      modalError = $_("settings__storage_modal_error_reset_config");
    } finally {
      resettingConfig = false;
    }
  }

  async function loadStorageObjects() {
    loadingObjects = true;
    pageError = "";

    try {
      const response = await api.admin.listStorageObjects(currentPage, pageSize);
      const data = await response.json();

      if (!response.ok) {
        pageError = $_("settings__storage_modal_error_load_objects");
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
      pageError = $_("settings__storage_modal_error_load_objects");
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
    pageError = "";

    try {
      const response = await api.admin.deleteStorageObject(
        encodeURIComponent(item.key),
      );
      const data = await response.json();

      if (!response.ok) {
        pageError = $_("settings__storage_modal_error_delete_object");
        return;
      }

      await Promise.all([loadStorageStats(), loadStorageObjects()]);
    } catch (error) {
      console.error("Failed to delete storage object:", error);
      pageError = $_("settings__storage_modal_error_delete_object");
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

    void loadStorageConfig();
    void loadStorageStats();
    void loadStorageObjects();
  });

  $: normalizedUsedGb =
    Number.isFinite(usedGb) && usedGb >= 0 ? Number(usedGb) : 0;
  $: storageProviderLabel =
    storageProvider === "env"
      ? $_("settings__storage_provider_rustfs")
      : $_("settings__storage_provider_s3");
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
    <div class="flex items-start justify-between gap-4">
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
      <button
        type="button"
        class="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-gray-200 bg-white text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-200 dark:hover:bg-gray-800"
        on:click={openProviderModal}
        aria-label={$_("settings__storage_settings_open")}
        title={$_("settings__storage_settings_open")}
      >
        <Settings2 size={18} />
      </button>
    </div>

    <div class="mt-5 grid gap-3 lg:grid-cols-2">
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_provider_label")}
        </p>
        <p class="mt-2 text-sm font-semibold text-gray-900 dark:text-white">
          {storageProviderLabel}
        </p>
      </div>
      <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
          {$_("settings__storage_field_bucket")}
        </p>
        <p class="mt-2 font-mono text-sm text-gray-900 dark:text-white">
          {bucketName}
        </p>
      </div>
      {#if storageProvider === "s3"}
        <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60">
          <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
            {$_("settings__storage_region_label")}
          </p>
          <p class="mt-2 text-sm text-gray-900 dark:text-white">
            {storageRegion}
          </p>
        </div>
      {/if}
      {#if storageProvider === "env"}
        <div class="rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/60 lg:col-span-2">
          <p class="text-xs font-semibold uppercase tracking-[0.2em] text-gray-500 dark:text-gray-400">
            {$_("settings__storage_rustfs_url_label")}
          </p>
          <p class="mt-2 text-sm text-gray-900 dark:text-white">
            {storageEndpoint}
          </p>
        </div>
      {/if}
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
        <iframe
          src={buildFileUrl(previewKey)}
          title={previewKey}
          class="h-full w-full border-0 bg-white dark:bg-gray-950"
        ></iframe>
      </div>
    </div>
  </div>
{/if}

{#if providerModalOpen}
  <div class="fixed inset-0 z-[1150] flex items-center justify-center p-4">
    <button
      type="button"
      class="absolute inset-0 bg-slate-950/75 backdrop-blur-sm"
      on:click={closeProviderModal}
      aria-label={$_("settings__storage_settings_close")}
    ></button>

    <div class="relative z-[1] w-full max-w-3xl overflow-hidden rounded-[28px] border border-slate-700/80 bg-slate-900 text-white shadow-[0_30px_100px_rgba(15,23,42,0.6)]">
      <div class="bg-[radial-gradient(circle_at_top_left,_rgba(56,189,248,0.18),_transparent_35%),linear-gradient(135deg,_#0f172a_0%,_#111827_55%,_#172554_100%)] px-6 py-5 sm:px-7">
        <div class="flex items-start justify-between gap-4">
          <div class="flex items-center gap-3">
            <div class="rounded-2xl bg-sky-500/10 p-3 text-sky-300 ring-1 ring-sky-400/20">
              <Settings2 size={18} />
            </div>
            <div>
              <h3 class="text-lg font-bold">{$_("settings__storage_modal_title")}</h3>
              <div class="text-sm text-slate-300">
                {#if draftStorageProvider === "env"}
                  <p>
                    {$_("settings__storage_modal_rustfs_help_before")}
                    <a
                      href={rustfsDocsUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="font-medium text-sky-300 underline decoration-sky-400/50 underline-offset-4 transition hover:text-sky-200"
                    >
                      {$_("settings__storage_modal_rustfs_docs")}
                    </a>
                    {$_("settings__storage_modal_rustfs_help_and")}
                    <a
                      href={rustfsSdkUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="font-medium text-sky-300 underline decoration-sky-400/50 underline-offset-4 transition hover:text-sky-200"
                    >
                      {$_("settings__storage_modal_rustfs_sdk")}
                    </a>
                  </p>
                {:else}
                  <p>
                    {$_("settings__storage_modal_s3_help_before")}
                    <a
                      href={awsS3DocsUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="font-medium text-sky-300 underline decoration-sky-400/50 underline-offset-4 transition hover:text-sky-200"
                    >
                      {$_("settings__storage_modal_s3_docs")}
                    </a>
                    {$_("settings__storage_modal_rustfs_help_and")}
                    <a
                      href={awsS3PolicyDocsUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="font-medium text-sky-300 underline decoration-sky-400/50 underline-offset-4 transition hover:text-sky-200"
                    >
                      {$_("settings__storage_modal_s3_policy")}
                    </a>
                  </p>
                {/if}
              </div>
            </div>
          </div>
          <button
            type="button"
            class="inline-flex h-10 w-10 min-h-10 min-w-10 shrink-0 items-center justify-center rounded-full border border-slate-600/70 bg-slate-950/55 text-slate-300 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] transition hover:border-slate-500 hover:bg-slate-900 hover:text-white"
            on:click={closeProviderModal}
            aria-label={$_("settings__storage_settings_close")}
          >
            <X size={16} strokeWidth={2.25} />
          </button>
        </div>
      </div>

      <div class="px-6 py-6 sm:px-7">
        {#if modalError}
          <div class="mb-4 rounded-2xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-100">
            {modalError}
          </div>
        {/if}

        {#if modalSuccess}
          <div class="mb-4 rounded-2xl border border-emerald-500/20 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-100">
            {modalSuccess}
          </div>
        {/if}

        <div class="grid gap-4 md:grid-cols-2">
          <label class="flex flex-col gap-2 text-sm font-medium text-slate-200">
            <span>{$_("settings__storage_provider_label")}</span>
            <select bind:value={draftStorageProvider} class="rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 text-white outline-none transition focus:border-sky-400">
              <option value="env">{$_("settings__storage_provider_rustfs")}</option>
              <option value="s3">{$_("settings__storage_provider_s3")}</option>
            </select>
          </label>

          <label class="flex flex-col gap-2 text-sm font-medium text-slate-200">
            <span>{$_("settings__storage_field_bucket")}</span>
            <input bind:value={draftBucketName} class="rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 text-white outline-none transition focus:border-sky-400" />
          </label>

          {#if draftStorageProvider === "s3"}
            <label class="flex flex-col gap-2 text-sm font-medium text-slate-200 md:col-span-2">
              <span>{$_("settings__storage_region_label")}</span>
              <input bind:value={draftStorageRegion} class="rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 text-white outline-none transition focus:border-sky-400" />
            </label>
          {:else}
            <label class="flex flex-col gap-2 text-sm font-medium text-slate-200 md:col-span-2">
              <span>{$_("settings__storage_rustfs_url_label")}</span>
              <input bind:value={draftStorageEndpoint} class="rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 text-white outline-none transition focus:border-sky-400" />
            </label>
          {/if}

          <label class="flex flex-col gap-2 text-sm font-medium text-slate-200">
            <span>{$_("settings__storage_access_key_label")}</span>
            <div class="relative">
              <input
                bind:value={draftStorageAccessKey}
                type={showAccessKey ? "text" : "password"}
                class="w-full rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 pr-12 text-white outline-none transition focus:border-sky-400"
              />
              <button
                type="button"
                class="absolute right-3 top-1/2 inline-flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full text-slate-400 transition hover:bg-slate-800 hover:text-white"
                aria-label={showAccessKey
                  ? $_("settings__storage_modal_hide_access_key")
                  : $_("settings__storage_modal_show_access_key")}
                on:click={() => (showAccessKey = !showAccessKey)}
              >
                {#if showAccessKey}
                  <EyeOff size={16} />
                {:else}
                  <Eye size={16} />
                {/if}
              </button>
            </div>
          </label>

          <label class="flex flex-col gap-2 text-sm font-medium text-slate-200">
            <span>{$_("settings__storage_secret_key_label")}</span>
            <div class="relative">
              <input
                bind:value={draftStorageSecretKey}
                type={showSecretKey ? "text" : "password"}
                class="w-full rounded-2xl border border-slate-700 bg-slate-950/70 px-4 py-3 pr-12 text-white outline-none transition focus:border-sky-400"
              />
              <button
                type="button"
                class="absolute right-3 top-1/2 inline-flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full text-slate-400 transition hover:bg-slate-800 hover:text-white"
                aria-label={showSecretKey
                  ? $_("settings__storage_modal_hide_secret_key")
                  : $_("settings__storage_modal_show_secret_key")}
                on:click={() => (showSecretKey = !showSecretKey)}
              >
                {#if showSecretKey}
                  <EyeOff size={16} />
                {:else}
                  <Eye size={16} />
                {/if}
              </button>
            </div>
          </label>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <div class="mr-auto">
            <button
              type="button"
              class="rounded-2xl border border-slate-600/70 bg-slate-900/70 px-5 py-3 text-sm font-semibold text-slate-200 transition hover:border-slate-500 hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
              on:click={resetStorageConfig}
              disabled={savingConfig || resettingConfig}
            >
              {#if resettingConfig}
                {$_("settings__storage_modal_reset")}
              {:else}
                {$_("settings__storage_modal_reset")}
              {/if}
            </button>
          </div>
          <button
            type="button"
            class="rounded-2xl border border-slate-700 bg-slate-900/70 px-5 py-3 text-sm font-semibold text-slate-200 transition hover:bg-slate-800"
            on:click={closeProviderModal}
            disabled={savingConfig || resettingConfig}
          >
            {$_("common.cancel")}
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-2xl border border-sky-400/30 bg-[linear-gradient(135deg,_rgba(59,130,246,0.95),_rgba(14,165,233,0.9))] px-5 py-3 text-sm font-semibold text-white shadow-[0_14px_34px_rgba(37,99,235,0.28)] transition hover:border-sky-300/40 hover:bg-[linear-gradient(135deg,_rgba(96,165,250,0.98),_rgba(56,189,248,0.95))] disabled:cursor-not-allowed disabled:border-slate-700 disabled:bg-slate-700 disabled:text-slate-400 disabled:shadow-none"
            disabled={savingConfig || resettingConfig}
            on:click={saveStorageConfig}
          >
            {#if savingConfig}
              <LoaderCircle size={16} class="animate-spin" />
            {/if}
            {$_("settings__storage_modal_save")}
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
