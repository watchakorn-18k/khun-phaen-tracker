import { browser } from "$app/environment";
import { base } from "$app/paths";
import { derived, get, writable } from "svelte/store";
import type { Assignee } from "$lib/types";
import { user, type User } from "$lib/stores/auth";
import { dataChanged } from "$lib/stores/realtime";

export type NotificationSourceType = "task" | "test_case";

export type AssignmentNotificationMeta = {
  source_type: NotificationSourceType;
  source_id: string;
  source_title: string;
  workspace_id: string;
  suite_id?: string;
  assignment_ids: string[];
  recipient_user_ids: string[];
  actor_user_id?: string;
  actor_name?: string;
  created_at: string;
};

export type AppNotification = {
  id: string;
  sourceType: NotificationSourceType;
  sourceId: string;
  sourceTitle: string;
  workspaceId: string;
  suiteId?: string;
  actorName: string;
  message: string;
  href: string;
  createdAt: string;
  readAt: string | null;
};

const notifications = writable<AppNotification[]>([]);
let unsubscribeRealtime: (() => void) | null = null;
let unsubscribeUser: (() => void) | null = null;
let activeUserKey = "";

export const notificationItems = notifications;
export const unreadNotificationCount = derived(notifications, ($items) =>
  $items.filter((item) => !item.readAt).length,
);

export function getUserDisplayName(currentUser: User | null): string {
  if (!currentUser) return "Unknown user";
  return (
    currentUser.profile?.nickname ||
    [currentUser.profile?.first_name, currentUser.profile?.last_name]
      .filter(Boolean)
      .join(" ")
      .trim() ||
    currentUser.email?.split("@")[0] ||
    "Unknown user"
  );
}

export function resolveRecipientUserIds(
  assignees: Assignee[],
  assigneeIds: Array<string | number | undefined | null>,
): string[] {
  const selected = new Set(
    assigneeIds
      .filter((id): id is string | number => id !== undefined && id !== null)
      .map(String),
  );

  return Array.from(
    new Set(
      assignees
        .filter((assignee) => assignee.id !== undefined && selected.has(String(assignee.id)))
        .map((assignee) => assignee.user_id)
        .filter((userId): userId is string => Boolean(userId)),
    ),
  );
}

export function getAssignmentIds(
  nextIds: Array<string | number | undefined | null> = [],
  previousIds: Array<string | number | undefined | null> = [],
): string[] {
  const previous = new Set(
    previousIds
      .filter((id): id is string | number => id !== undefined && id !== null)
      .map(String),
  );

  return Array.from(
    new Set(
      nextIds
        .filter((id): id is string | number => id !== undefined && id !== null)
        .map(String)
        .filter((id) => id !== "unassigned" && !previous.has(id)),
    ),
  );
}

export function shouldCreateNotification(
  meta: AssignmentNotificationMeta,
  currentUser: User | null,
): boolean {
  if (!currentUser) return false;

  const userIds = [currentUser.id, currentUser.user_id].filter(Boolean).map(String);
  const actorIds = meta.actor_user_id ? [String(meta.actor_user_id)] : [];
  if (actorIds.some((actorId) => userIds.includes(actorId))) return false;

  return meta.recipient_user_ids.some((recipientId) =>
    userIds.includes(String(recipientId)),
  );
}

export function buildNotification(
  meta: AssignmentNotificationMeta,
): AppNotification {
  const sourceLabel = meta.source_type === "task" ? "งาน" : "test case";
  const actorName = meta.actor_name || "Unknown user";
  const params = new URLSearchParams();
  const room = browser ? localStorage.getItem("sync-room-code") : "";
  if (room) params.set("room", room);

  let href = `${base}/workspace/${meta.workspace_id}/task/${meta.source_id}`;
  if (meta.source_type === "test_case") {
    if (meta.suite_id) params.set("suite", meta.suite_id);
    params.set("case", meta.source_id);
    href = `${base}/workspace/${meta.workspace_id}/test-cases/editor`;
  }

  const query = params.toString();

  return {
    id: `${meta.source_type}:${meta.source_id}:${meta.assignment_ids.join(",")}:${meta.created_at}`,
    sourceType: meta.source_type,
    sourceId: meta.source_id,
    sourceTitle: meta.source_title,
    workspaceId: meta.workspace_id,
    suiteId: meta.suite_id,
    actorName,
    message: `${actorName} เพิ่ม${sourceLabel} "${meta.source_title}" ให้คุณ`,
    href: query ? `${href}?${query}` : href,
    createdAt: meta.created_at,
    readAt: null,
  };
}

export function initNotifications() {
  if (!browser) return;
  if (!unsubscribeUser) {
    unsubscribeUser = user.subscribe((currentUser) => {
      const nextKey = currentUser?.id || currentUser?.user_id || "";
      if (nextKey === activeUserKey) return;
      activeUserKey = nextKey;
      notifications.set(readStoredNotifications(nextKey));
    });
  }

  if (!unsubscribeRealtime) {
    unsubscribeRealtime = dataChanged.subscribe((event) => {
      const meta = event?.data?._notification as AssignmentNotificationMeta | undefined;
      if (meta) ingestAssignmentNotification(meta);
    });
  }
}

export function ingestAssignmentNotification(meta: AssignmentNotificationMeta) {
  if (!shouldCreateNotification(meta, get(user))) return;
  upsertNotification(buildNotification(meta));
}

export function destroyNotifications() {
  unsubscribeRealtime?.();
  unsubscribeUser?.();
  unsubscribeRealtime = null;
  unsubscribeUser = null;
}

export function markNotificationRead(id: string) {
  updateStored((items) =>
    items.map((item) =>
      item.id === id && !item.readAt
        ? { ...item, readAt: new Date().toISOString() }
        : item,
    ),
  );
}

export function markAllNotificationsRead() {
  const now = new Date().toISOString();
  updateStored((items) =>
    items.map((item) => (item.readAt ? item : { ...item, readAt: now })),
  );
}

export function clearNotificationsForTest() {
  notifications.set([]);
  activeUserKey = "";
}

function upsertNotification(notification: AppNotification) {
  updateStored((items) => {
    if (items.some((item) => item.id === notification.id)) return items;
    return [notification, ...items].slice(0, 100);
  });
}

function updateStored(updater: (items: AppNotification[]) => AppNotification[]) {
  const currentUser = get(user);
  const key = currentUser?.id || currentUser?.user_id || "";
  if (!key) return;

  const next = updater(get(notifications));
  notifications.set(next);
  if (browser) {
    localStorage.setItem(storageKey(key), JSON.stringify(next));
  }
}

function readStoredNotifications(userKey: string): AppNotification[] {
  if (!browser || !userKey) return [];
  try {
    const raw = localStorage.getItem(storageKey(userKey));
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function storageKey(userKey: string) {
  return `khun-phaen-notifications:${userKey}`;
}
