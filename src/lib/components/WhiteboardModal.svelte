<script lang="ts">
  import { createEventDispatcher, onDestroy, tick } from "svelte";
  import {
    Wifi,
    WifiOff,
    Download,
    Trash2,
    X,
    LoaderCircle,
    Maximize2,
    Minimize2,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { requestConfirm } from "$lib/stores/confirmStore";

  type Snapshot = Record<string, unknown>;
  type BridgeMode = "offline" | "sync";
  type BridgeSyncStatus = "offline" | "connecting" | "online" | "error";

  interface MountedExcalidrawBridge {
    unmount: () => void;
    loadSnapshot: (snapshot: Snapshot) => boolean;
    exportPng: () => Promise<string | null>;
    clearCurrentPage: () => void;
  }

  interface ExcalidrawBridgeModule {
    mountExcalidrawBridge: (options: {
      target: HTMLElement;
      mode: BridgeMode;
      initialSnapshot: Snapshot | null;
      onUserSnapshot: (snapshot: Snapshot) => void;
      onSyncStatus?: (status: BridgeSyncStatus, message: string) => void;
      syncRoomId?: string;
      syncHost?: string;
      isDarkMode?: boolean;
    }) => MountedExcalidrawBridge;
  }

  type ConnectionState = "offline" | "connecting" | "online" | "error";

  const SYNC_ROOM_KEY = "sync-room-code";
  const SNAPSHOT_KEY = "excalidraw-snapshot-v1";
  const SYNC_HOST_KEY = "whiteboard-sync-host";
  const DEFAULT_SYNC_HOST = "https://excalidraw.com"; // Updated default host placeholder

  const dispatch = createEventDispatcher<{
    close: void;
    notify: { message: string; type: "success" | "error" };
  }>();

  export let open = false;

  let boardMountEl: HTMLDivElement | null = null;
  let bridge: MountedExcalidrawBridge | null = null;
  let bridgeModule: ExcalidrawBridgeModule | null = null;
  let connectionState: ConnectionState = "offline";
  let statusMessage = $_("whiteboard__status_offline");
  let isBooting = false;
  let active = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let latestSnapshotRaw: string | null = null;
  let mode: BridgeMode = "offline";
  let roomCode = "";
  let syncHost = DEFAULT_SYNC_HOST;
  let isFullscreen = false;

  $: if (open && !active) {
    active = true;
    void startWhiteboard();
  }

  $: if (!open && active) {
    active = false;
    stopWhiteboard();
  }

  onDestroy(() => {
    stopWhiteboard();
  });

  function readSnapshotFromStorage(): Snapshot | null {
    if (typeof localStorage === "undefined") return null;
    const raw = localStorage.getItem(SNAPSHOT_KEY);
    if (!raw) return null;
    latestSnapshotRaw = raw;
    try {
      return JSON.parse(raw) as Snapshot;
    } catch {
      localStorage.removeItem(SNAPSHOT_KEY);
      latestSnapshotRaw = null;
      return null;
    }
  }

  function persistSnapshot(snapshot: Snapshot) {
    try {
      const serialized = JSON.stringify(snapshot);
      latestSnapshotRaw = serialized;

      if (saveTimer) clearTimeout(saveTimer);
      saveTimer = setTimeout(() => {
        localStorage.setItem(SNAPSHOT_KEY, serialized);
      }, 500); // Slightly longer debounce for excalidraw
    } catch (error) {
      console.warn("Failed to persist whiteboard snapshot", error);
    }
  }

  function handleUserSnapshot(snapshot: Snapshot) {
    persistSnapshot(snapshot);
  }

  function resolveModeFromStorage() {
    const savedRoom =
      localStorage.getItem(SYNC_ROOM_KEY)?.trim().toUpperCase() ?? "";
    const configuredSyncHost = localStorage.getItem(SYNC_HOST_KEY)?.trim();

    roomCode = savedRoom;
    syncHost =
      configuredSyncHost && configuredSyncHost.length > 0
        ? configuredSyncHost
        : DEFAULT_SYNC_HOST;

    if (roomCode) {
      mode = "sync";
      connectionState = "connecting";
      statusMessage = $_("whiteboard__status_connecting", {
        values: { room: roomCode },
      });
      return;
    }

    mode = "offline";
    connectionState = "offline";
    statusMessage = $_("whiteboard__status_no_room");
  }

  function handleSyncStatus(status: BridgeSyncStatus, message: string) {
    if (mode !== "sync") return;
    connectionState = status;

    if (status === "online") {
      statusMessage = $_("whiteboard__status_online", {
        values: { room: roomCode },
      });
    } else if (status === "connecting") {
      statusMessage = $_("whiteboard__status_connecting", {
        values: { room: roomCode },
      });
    } else if (status === "error") {
      statusMessage = $_("whiteboard__status_error_msg", {
        values: { message },
      });
    } else if (status === "offline") {
      statusMessage = $_("whiteboard__status_offline");
    } else {
      statusMessage = message;
    }
  }

  async function ensureBridge() {
    if (!boardMountEl) return;
    if (!bridgeModule) {
      const mod = await import("$lib/components/excalidrawBridge");
      bridgeModule = mod as unknown as ExcalidrawBridgeModule;
    }
    if (bridge) return;
    const forceDark =
      typeof document !== "undefined" &&
      document.documentElement.classList.contains("dark");

    bridge = bridgeModule.mountExcalidrawBridge({
      target: boardMountEl,
      mode,
      initialSnapshot: readSnapshotFromStorage(),
      onUserSnapshot: handleUserSnapshot,
      onSyncStatus: handleSyncStatus,
      isDarkMode: forceDark,
      ...(mode === "sync" ? { syncRoomId: roomCode, syncHost } : {}),
    });
  }

  async function startWhiteboard() {
    isBooting = true;
    try {
      resolveModeFromStorage();
      await tick();
      await ensureBridge();
    } finally {
      isBooting = false;
    }
  }

  function stopWhiteboard() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }

    if (bridge) {
      bridge.unmount();
      bridge = null;
    }

    mode = "offline";
    connectionState = "offline";
    statusMessage = $_("whiteboard__status_offline");
    roomCode = "";
  }

  async function handleExportPng() {
    if (!bridge) return;
    const dataUrl = await bridge.exportPng();
    if (!dataUrl) {
      dispatch("notify", {
        message: $_("whiteboard__notify_export_error"),
        type: "error",
      });
      return;
    }
    const a = document.createElement("a");
    a.href = dataUrl;
    a.download = `whiteboard-${new Date().toISOString().replace(/[:.]/g, "-")}.png`;
    a.click();
    dispatch("notify", {
      message: $_("whiteboard__notify_export_success"),
      type: "success",
    });
  }

  async function handleClearBoard() {
    const confirmed = await requestConfirm({
      title: $_("common.confirm"),
      message: $_("whiteboard__confirm_clear"),
      type: "danger",
    });
    if (!confirmed) return;
    bridge?.clearCurrentPage();
    localStorage.removeItem(SNAPSHOT_KEY);
    latestSnapshotRaw = null;
    dispatch("notify", {
      message: $_("whiteboard__notify_clear_success"),
      type: "success",
    });
  }
</script>

{#if open}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-[20000] bg-black/60 backdrop-blur-sm flex items-center justify-center transition-all duration-300"
    class:p-2={!isFullscreen}
    class:sm:p-4={!isFullscreen}
    class:md:p-6={!isFullscreen}
    class:pt-20={!isFullscreen}
    class:sm:pt-20={!isFullscreen}
    class:md:pt-6={!isFullscreen}
    on:click|self={() => dispatch("close")}
  >
    <div
      class="bg-white dark:bg-gray-900 flex flex-col overflow-hidden transition-all duration-300 ease-in-out"
      class:w-full={isFullscreen}
      class:h-full={isFullscreen}
      class:max-w-7xl={!isFullscreen}
      class:rounded-lg={!isFullscreen}
      class:sm:rounded-xl={!isFullscreen}
      class:md:rounded-2xl={!isFullscreen}
      class:shadow-2xl={!isFullscreen}
      class:border={!isFullscreen}
      class:border-gray-200={!isFullscreen}
      class:dark:border-gray-700={!isFullscreen}
      style={!isFullscreen ? "width: 100%; height: calc(100vh - 5rem);" : ""}
    >
      <div
        class="px-3 sm:px-4 md:px-5 py-2.5 sm:py-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between gap-2"
      >
        <div class="min-w-0 flex-1">
          <h3
            class="text-sm sm:text-base md:text-lg font-semibold text-gray-900 dark:text-white truncate"
          >
            {$_("whiteboard__title")}
          </h3>
          <div
            class="text-[10px] sm:text-xs text-gray-500 dark:text-gray-400 mt-0.5 flex items-center gap-1 sm:gap-1.5"
          >
            {#if connectionState === "online"}
              <Wifi size={12} class="text-emerald-500 shrink-0" />
            {:else if connectionState === "connecting"}
              <LoaderCircle
                size={12}
                class="animate-spin text-amber-500 shrink-0"
              />
            {:else if connectionState === "error"}
              <WifiOff size={12} class="text-red-500 shrink-0" />
            {:else}
              <WifiOff size={12} class="text-gray-400 shrink-0" />
            {/if}
            <span class="truncate">{statusMessage}</span>
          </div>
        </div>

        <div class="flex items-center gap-1.5 sm:gap-2 shrink-0">
          <button
            on:click={() => (isFullscreen = !isFullscreen)}
            class="px-2 sm:px-3 py-1.5 sm:py-2 text-xs sm:text-sm rounded-md sm:rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 sm:gap-1.5"
            title={isFullscreen
              ? $_("whiteboard__mode_mini")
              : $_("whiteboard__mode_fullscreen")}
          >
            {#if isFullscreen}
              <Minimize2 size={15} />
              <span class="hidden md:inline">{$_("whiteboard__mode_mini")}</span
              >
            {:else}
              <Maximize2 size={15} />
              <span class="hidden md:inline"
                >{$_("whiteboard__mode_fullscreen")}</span
              >
            {/if}
          </button>
          <button
            on:click={handleExportPng}
            class="px-2 sm:px-3 py-1.5 sm:py-2 text-xs sm:text-sm rounded-md sm:rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 sm:gap-1.5"
            title={$_("whiteboard__btn_export")}
          >
            <Download size={15} />
            <span class="hidden md:inline">{$_("whiteboard__btn_export")}</span>
          </button>
          <button
            on:click={handleClearBoard}
            class="px-2 sm:px-3 py-1.5 sm:py-2 text-xs sm:text-sm rounded-md sm:rounded-lg border border-red-300 dark:border-red-800 text-red-600 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors flex items-center gap-1 sm:gap-1.5"
            title={$_("whiteboard__btn_clear")}
          >
            <Trash2 size={15} />
            <span class="hidden md:inline">{$_("whiteboard__btn_clear")}</span>
          </button>
          <button
            on:click={() => dispatch("close")}
            class="w-8 h-8 sm:w-9 sm:h-9 flex items-center justify-center rounded-md sm:rounded-lg border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors shrink-0"
            title={$_("whiteboard__btn_close")}
          >
            <X size={16} />
          </button>
        </div>
      </div>

      <div class="flex-1 min-h-0 relative">
        {#if isBooting}
          <div
            class="absolute inset-0 z-10 bg-white/80 dark:bg-gray-900/80 flex items-center justify-center"
          >
            <div
              class="flex items-center gap-2 text-xs sm:text-sm text-gray-600 dark:text-gray-400"
            >
              <LoaderCircle size={14} class="sm:w-4 sm:h-4 animate-spin" />
              <span>{$_("whiteboard__loading")}</span>
            </div>
          </div>
        {/if}
        <div
          class="whiteboard-excalidraw-host w-full h-full"
          bind:this={boardMountEl}
        ></div>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark) {
    --icon-fill-color: #e5e7eb;
    --color-on-surface: #e5e7eb;
    --text-primary-color: #e5e7eb;
    --color-disabled: #8b8f9c;
    --keybinding-color: #9ca3af;
    --island-bg-color: #1f2028;
    --button-gray-1: #2d2f3a;
    --button-gray-2: #383a46;
    --button-gray-3: #444653;
    --button-hover-bg: #343642;
    --default-border-color: #3f4250;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark button),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark button svg),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .ToolIcon__icon),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .ToolIcon__icon svg),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .dropdown-menu-button),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .dropdown-menu-button svg) {
    color: var(--icon-fill-color);
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item-base
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item__text
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item__shortcut
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-group-title
    ) {
    color: #e5e7eb;
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item-base[aria-disabled="true"]
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item-base[disabled]
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item-base[aria-disabled="true"]
        *
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .dropdown-menu
        .dropdown-menu-item-base[disabled]
        *
    ) {
    color: var(--color-disabled);
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon
        .ToolIcon_type_radio:checked
        + .ToolIcon__icon
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon
        .ToolIcon_type_checkbox:checked
        + .ToolIcon__icon
    ) {
    --color-on-primary-container: #ffffff;
    background: #4f46a5;
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon
        .ToolIcon_type_radio:checked
        + .ToolIcon__icon
        svg
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon
        .ToolIcon_type_checkbox:checked
        + .ToolIcon__icon
        svg
    ) {
    color: #ffffff;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark button:disabled),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark button:disabled svg),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon__icon[aria-disabled="true"]
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .ToolIcon__icon[aria-disabled="true"]
        svg
    ) {
    color: var(--color-disabled);
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .context-menu),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .context-menu button),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item
        .context-menu-item__label
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item
        .context-menu-item__shortcut
    ) {
    color: #e5e7eb;
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item[aria-disabled="true"]
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item[disabled]
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item[aria-disabled="true"]
        *
    ),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item[disabled]
        *
    ) {
    color: #8b8f9c;
  }

  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .context-menu-item.dangerous
        .context-menu-item__label
    ) {
    color: #ff6b6b;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .panelColumn h3),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .panelColumn legend),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .panelColumn
        .control-label
    ),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .zero-label),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .value-bubble),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .drag-input-label),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .color-picker__heading),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .color-picker__input-label
    ) {
    color: #e5e7eb;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .zoom-actions),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .undo-redo-buttons) {
    background-color: #1f2028;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .zoom-button),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .undo-redo-buttons
        button
    ) {
    background-color: #252631 !important;
    color: #e5e7eb !important;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .reset-zoom-button),
  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .zoom-button svg),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .undo-redo-buttons
        button
        svg
    ) {
    color: #e5e7eb !important;
  }

  :global(.whiteboard-excalidraw-host .excalidraw.theme--dark .zoom-button:hover),
  :global(
      .whiteboard-excalidraw-host
        .excalidraw.theme--dark
        .undo-redo-buttons
        button:hover
    ) {
    background-color: #343642 !important;
  }
</style>
