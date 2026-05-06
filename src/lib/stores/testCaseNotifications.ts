import type { Assignee } from "$lib/types";
import { get } from "svelte/store";
import { user } from "$lib/stores/auth";
import { broadcastChange } from "$lib/stores/realtime";
import {
  getAssignmentIds,
  getUserDisplayName,
  ingestAssignmentNotification,
  resolveRecipientUserIds,
  saveNotificationForRecipients,
  setCurrentUserAssigneeIds,
  type AssignmentNotificationMeta,
} from "$lib/stores/notifications";

type TestCaseNotificationInput = {
  id: string;
  title: string;
  workspaceId: string;
  suiteId?: string | null;
  assignees: Assignee[];
  nextAssigneeIds: string[];
  previousAssigneeIds?: string[];
  action: "create" | "update";
};

export function broadcastTestCaseAssignment(input: TestCaseNotificationInput) {
  const assignmentIds = getAssignmentIds(
    input.nextAssigneeIds,
    input.previousAssigneeIds || [],
  );
  if (assignmentIds.length === 0) return;
  const recipientUserIds = resolveRecipientUserIds(input.assignees, assignmentIds);
  const actor = get(user);
  const myAssignee = input.assignees.find(
    (a) => a.user_id && (a.user_id === actor?.id || a.user_id === actor?.user_id),
  );
  if (myAssignee?.id) setCurrentUserAssigneeIds([String(myAssignee.id)]);
  const notification: AssignmentNotificationMeta | undefined =
    assignmentIds.length > 0
      ? {
          source_type: "test_case",
          source_id: input.id,
          source_title: input.title || "Untitled test case",
          workspace_id: input.workspaceId,
          suite_id: input.suiteId || undefined,
          assignment_ids: assignmentIds,
          recipient_user_ids: recipientUserIds,
          actor_user_id: actor?.id || actor?.user_id,
          actor_name: getUserDisplayName(actor),
          created_at: new Date().toISOString(),
        }
      : undefined;

  if (notification) {
    void saveNotificationForRecipients(notification);
    ingestAssignmentNotification(notification);
  }

  broadcastChange("test_case", input.action, input.id, {
    id: input.id,
    title: input.title,
    suite_id: input.suiteId || undefined,
    assign_dev: input.nextAssigneeIds[0] || "unassigned",
    _notification: notification,
  });
}
