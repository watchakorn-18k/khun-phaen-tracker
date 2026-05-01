import { createElement, useEffect, useRef } from "react";
import { createRoot, type Root } from "react-dom/client";
import { flushSync } from "react-dom";
import { Excalidraw, exportToBlob } from "@excalidraw/excalidraw";
import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import type { AppState, ExcalidrawImperativeAPI } from "@excalidraw/excalidraw/types";

// Excalidraw CSS is required
import "@excalidraw/excalidraw/index.css";

export type Snapshot = {
  elements: readonly ExcalidrawElement[];
  appState: Partial<AppState>;
  files: Record<string, any>;
};

export type ExcalidrawBridgeMode = "offline" | "sync";
export type ExcalidrawBridgeSyncStatus =
  | "offline"
  | "connecting"
  | "online"
  | "error";

interface BaseBridgeProps {
  initialSnapshot: Snapshot | null;
  onApiReady: (api: ExcalidrawImperativeAPI) => void;
  onUserSnapshot: (snapshot: Snapshot) => void;
  isDarkMode?: boolean;
}

interface OfflineCanvasProps extends BaseBridgeProps {
  mode: "offline";
}

// Note: Excalidraw doesn't have a built-in sync like tldraw.
// For now, we will treat sync mode as offline but maybe with a status message.
interface SyncCanvasProps extends BaseBridgeProps {
  mode: "sync";
  syncRoomId: string;
  syncHost: string;
  onSyncStatus: (status: ExcalidrawBridgeSyncStatus, message: string) => void;
}

type BridgeAppProps = OfflineCanvasProps | SyncCanvasProps;

const DARK_CANVAS_BACKGROUND = "#121212";
const DARK_THEME_STROKE = "#e9ecef";
const LIGHT_CANVAS_BACKGROUND = "#ffffff";
const LIGHT_THEME_STROKE = "#1e1e1e";

function getThemeAppState(isDarkMode?: boolean): Partial<AppState> {
  return {
    theme: isDarkMode ? "dark" : "light",
    currentItemStrokeColor: isDarkMode ? DARK_THEME_STROKE : LIGHT_THEME_STROKE,
    viewBackgroundColor: isDarkMode
      ? DARK_CANVAS_BACKGROUND
      : LIGHT_CANVAS_BACKGROUND,
    exportWithDarkMode: Boolean(isDarkMode),
  } as Partial<AppState>;
}

function normalizeSnapshotForTheme(
  snapshot: Snapshot | null,
  isDarkMode?: boolean,
): Snapshot {
  const appState = snapshot?.appState ?? {};

  return {
    elements: snapshot?.elements ?? [],
    appState: {
      ...appState,
      collaborators:
        appState.collaborators instanceof Map
          ? appState.collaborators
          : new Map(),
      followedBy: appState.followedBy instanceof Set ? appState.followedBy : new Set(),
      ...getThemeAppState(isDarkMode),
    },
    files: snapshot?.files ?? {},
  };
}

function BridgeApp(props: BridgeAppProps) {
  const initializedRef = useRef(false);
  const initialDataRef = useRef(
    normalizeSnapshotForTheme(props.initialSnapshot, props.isDarkMode),
  );

  useEffect(() => {
    if (props.mode === "sync") {
      props.onSyncStatus("offline", "Excalidraw does not support built-in sync yet. Operating in offline mode.");
    }
  }, [props.mode]);

  const onChange = (elements: readonly ExcalidrawElement[], appState: AppState, files: Record<string, any>) => {
    if (!initializedRef.current) return;
    props.onUserSnapshot({ elements, appState, files });
  };

  return createElement("div", {
    style: { width: "100%", height: "100%", position: "relative" },
    children: createElement(Excalidraw, {
      excalidrawAPI: (api: ExcalidrawImperativeAPI) => {
        props.onApiReady(api);
        initializedRef.current = true;
      },
      initialData: initialDataRef.current,
      onChange,
      theme: props.isDarkMode ? "dark" : "light",
      UIOptions: {
        canvasActions: {
          loadScene: true,
          export: { saveFileToDisk: true },
          clearCanvas: true,
          toggleTheme: true,
          saveAsImage: true,
        },
      },
    }),
  });
}

export interface MountedExcalidrawBridge {
  unmount: () => void;
  loadSnapshot: (snapshot: Snapshot) => boolean;
  exportPng: () => Promise<string | null>;
  clearCurrentPage: () => void;
}

export interface MountExcalidrawBridgeOptions {
  target: HTMLElement;
  mode: ExcalidrawBridgeMode;
  initialSnapshot: Snapshot | null;
  onUserSnapshot: (snapshot: Snapshot) => void;
  onSyncStatus?: (status: ExcalidrawBridgeSyncStatus, message: string) => void;
  syncRoomId?: string;
  syncHost?: string;
  isDarkMode?: boolean;
}

export function mountExcalidrawBridge({
  target,
  mode,
  initialSnapshot,
  onUserSnapshot,
  onSyncStatus,
  syncRoomId,
  syncHost,
  isDarkMode,
}: MountExcalidrawBridgeOptions): MountedExcalidrawBridge {
  let root: Root | null = createRoot(target);
  let api: ExcalidrawImperativeAPI | null = null;
  let isUnmounted = false;

  const bridgeKey =
    mode === "sync"
      ? `sync-${syncRoomId}-${Date.now()}`
      : `offline-${Date.now()}`;

  const commonProps = {
    key: bridgeKey,
    initialSnapshot,
    onUserSnapshot,
    onApiReady: (nextApi: ExcalidrawImperativeAPI) => {
      if (!isUnmounted) api = nextApi;
    },
    isDarkMode,
  };

  if (mode === "sync") {
    root.render(
      createElement(BridgeApp, {
        ...commonProps,
        mode: "sync",
        syncRoomId: syncRoomId!,
        syncHost: syncHost!,
        onSyncStatus: onSyncStatus ?? (() => {}),
      })
    );
  } else {
    root.render(
      createElement(BridgeApp, {
        ...commonProps,
        mode: "offline",
      })
    );
  }

  const loadSnapshot = (snapshot: Snapshot): boolean => {
    if (!api) return false;
    try {
      const themedSnapshot = normalizeSnapshotForTheme(snapshot, isDarkMode);
      api.updateScene({
        elements: themedSnapshot.elements,
        appState: themedSnapshot.appState as any,
      });
      return true;
    } catch (error) {
      console.warn("Failed to apply whiteboard snapshot", error);
      return false;
    }
  };

  const exportPng = async (): Promise<string | null> => {
    if (!api) return null;
    const elements = api.getSceneElements();
    if (elements.length === 0) return null;
    
    try {
      const blob = await exportToBlob({
        elements,
        appState: api.getAppState(),
        files: api.getFiles(),
        mimeType: "image/png",
      });
      return URL.createObjectURL(blob);
    } catch (error) {
      console.error("Export failed", error);
      return null;
    }
  };

  const clearCurrentPage = () => {
    if (!api) return;
    api.updateScene({ elements: [] });
  };

  return {
    unmount: () => {
      if (!root || isUnmounted) return;
      isUnmounted = true;
      flushSync(() => {
        root?.unmount();
      });
      root = null;
      api = null;
    },
    loadSnapshot,
    exportPng,
    clearCurrentPage,
  };
}
