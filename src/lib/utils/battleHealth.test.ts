import { describe, it, expect } from "vitest";
import { calculateBattleHealthStats, isTaskOverdue } from "./battleHealth";
import type { Task } from "../types";

describe("Battle Health Logic", () => {
  const today = new Date("2024-01-15T12:00:00Z");

  describe("isTaskOverdue", () => {
    it("should return true for tasks past due date", () => {
      const task: Task = {
        title: "Overdue task",
        date: "2024-01-14",
        status: "todo",
        duration_minutes: 30,
        category: "อื่นๆ",
        notes: "",
      };
      expect(isTaskOverdue(task, today)).toBe(true);
    });

    it("should return false for tasks on due date", () => {
      const task: Task = {
        title: "Today task",
        date: "2024-01-15",
        status: "todo",
        duration_minutes: 30,
        category: "อื่นๆ",
        notes: "",
      };
      expect(isTaskOverdue(task, today)).toBe(false);
    });

    it("should return false for completed tasks even if past due date", () => {
      const task: Task = {
        title: "Done past task",
        date: "2024-01-10",
        status: "done",
        duration_minutes: 30,
        category: "อื่นๆ",
        notes: "",
      };
      expect(isTaskOverdue(task, today)).toBe(false);
    });
  });

  describe("calculateBattleHealthStats", () => {
    it("should calculate velocity and trend correctly", () => {
      const tasks: Task[] = [
        // Completed in last 7 days (8th to 14th)
        {
          title: "T1",
          status: "done",
          updated_at: "2024-01-10T10:00:00Z",
          duration_minutes: 30,
          category: "อื่นๆ",
          notes: "",
          date: "2024-01-10",
        },
        {
          title: "T2",
          status: "done",
          updated_at: "2024-01-12T10:00:00Z",
          duration_minutes: 30,
          category: "อื่นๆ",
          notes: "",
          date: "2024-01-12",
        },
        // Completed in previous 7 days (1st to 7th)
        {
          title: "T3",
          status: "done",
          updated_at: "2024-01-05T10:00:00Z",
          duration_minutes: 30,
          category: "อื่นๆ",
          notes: "",
          date: "2024-01-05",
        },
      ];

      const stats = calculateBattleHealthStats(tasks, today);
      expect(stats.doneTasksLast7Days).toBe(2);
      expect(stats.velocityTrend).toBe(1); // 2 - 1
      expect(stats.velocity).toBe("0.3"); // 2 / 7 = 0.28...
    });

    it("should calculate overdue ratio and morale", () => {
      const tasks: Task[] = [
        {
          title: "Active 1",
          status: "todo",
          date: "2024-01-16",
          duration_minutes: 30,
          category: "อื่นๆ",
          notes: "",
        },
        {
          title: "Overdue 1",
          status: "in-progress",
          date: "2024-01-10",
          duration_minutes: 30,
          category: "อื่นๆ",
          notes: "",
        },
      ];

      const stats = calculateBattleHealthStats(tasks, today);
      expect(stats.overdueCount).toBe(1);
      expect(stats.activeTasksCount).toBe(2);
      expect(stats.overdueRatio).toBe(50);
      expect(stats.morale).toBe("critical");
    });

    it("should return excellent morale if performance is high and no overdue", () => {
      const tasks: Task[] = Array(10).fill({
        title: "Done",
        status: "done",
        updated_at: "2024-01-10T10:00:00Z",
        duration_minutes: 30,
        category: "อื่นๆ",
        notes: "",
        date: "2024-01-10",
      });

      const stats = calculateBattleHealthStats(tasks, today);
      expect(stats.overdueRatio).toBe(0);
      expect(stats.morale).toBe("excellent");
    });
  });
});
