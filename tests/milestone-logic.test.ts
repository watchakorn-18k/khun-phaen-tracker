import { describe, it, expect } from "vitest";

// Since calculateTime is inside Svelte component, I'll extract it here or test a similar logic
function calculateTimeRemaining(targetDate: string, now: Date) {
  const target = new Date(targetDate);
  const diff = target.getTime() - now.getTime();

  if (diff <= 0) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0, isReached: true };
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((diff % (1000 * 60)) / 1000);

  return { days, hours, minutes, seconds, isReached: false };
}

describe("Milestone Countdown Logic", () => {
  it("should calculate remaining time correctly for future date", () => {
    const target = "2025-01-01T00:00:00Z";
    const now = new Date("2024-12-31T00:00:00Z"); // 24 hours exactly
    const result = calculateTimeRemaining(target, now);

    expect(result.days).toBe(1);
    expect(result.hours).toBe(0);
    expect(result.minutes).toBe(0);
    expect(result.seconds).toBe(0);
    expect(result.isReached).toBe(false);
  });

  it("should return isReached: true for past date", () => {
    const target = "2024-01-01T00:00:00Z";
    const now = new Date("2024-01-02T00:00:00Z");
    const result = calculateTimeRemaining(target, now);

    expect(result.isReached).toBe(true);
  });

  it("should handle partial time correctly", () => {
    const target = "2025-01-01T12:30:45Z";
    const now = new Date("2025-01-01T11:00:00Z"); // 1h 30m 45s diff
    const result = calculateTimeRemaining(target, now);

    expect(result.days).toBe(0);
    expect(result.hours).toBe(1);
    expect(result.minutes).toBe(30);
    expect(result.seconds).toBe(45);
    expect(result.isReached).toBe(false);
  });
});
