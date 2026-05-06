import { describe, expect, it } from "vitest";
import {
  buildNotification,
  getAssignmentIds,
  getUserDisplayName,
  resolveRecipientUserIds,
  shouldCreateNotification,
  type AssignmentNotificationMeta,
} from "./notifications";

describe("notifications", () => {
  it("detects only newly assigned assignee IDs", () => {
    expect(getAssignmentIds(["a", "b", "unassigned"], ["a"])).toEqual(["b"]);
  });

  it("maps assigned assignees to recipient user IDs", () => {
    expect(
      resolveRecipientUserIds(
        [
          { id: "dev-1", name: "Dev", user_id: "user-1" },
          { id: "dev-2", name: "Other" },
        ],
        ["dev-1", "dev-2"],
      ),
    ).toEqual(["user-1"]);
  });

  it("does not notify the actor when they assign themselves", () => {
    const meta: AssignmentNotificationMeta = {
      source_type: "task",
      source_id: "task-1",
      source_title: "Fix login",
      workspace_id: "ws-1",
      assignment_ids: ["dev-1"],
      recipient_user_ids: ["user-1"],
      actor_user_id: "user-1",
      actor_name: "Dev",
      created_at: "2026-05-07T00:00:00.000Z",
    };

    expect(
      shouldCreateNotification(meta, {
        id: "user-1",
        user_id: "user-uuid-1",
        email: "dev@example.com",
        role: "user",
      }),
    ).toBe(false);
  });

  it("builds a deep link for task notifications", () => {
    const item = buildNotification({
      source_type: "task",
      source_id: "task-1",
      source_title: "Fix login",
      workspace_id: "ws-1",
      assignment_ids: ["dev-1"],
      recipient_user_ids: ["user-2"],
      actor_user_id: "user-1",
      actor_name: "QA",
      created_at: "2026-05-07T00:00:00.000Z",
    });

    expect(item.message).toContain('QA เพิ่มงาน "Fix login" ให้คุณ');
    expect(item.href).toContain("/workspace/ws-1/task/task-1");
  });

  it("uses profile names before email for actor labels", () => {
    expect(
      getUserDisplayName({
        id: "user-1",
        user_id: "uuid-1",
        email: "dev@example.com",
        role: "user",
        profile: { first_name: "Ada", last_name: "Lovelace" },
      }),
    ).toBe("Ada Lovelace");
  });
});
