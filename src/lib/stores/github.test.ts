import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";
import { githubStore } from "./github";

describe("githubStore", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    // Reset store manually or re-import if possible.
    // For simplicity let's use the current store and just update it.
  });

  it("should initialize with default values", () => {
    const state = get(githubStore);
    expect(state.stars).toBe("...");
  });

  it("should update stars when updateCache is called", () => {
    githubStore.updateCache("42");
    const state = get(githubStore);
    expect(state.stars).toBe("42");
  });

  it("should fetch stars from API", async () => {
    const mockData = { stargazers_count: 123 };

    // Mock global fetch
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockData),
    });
    vi.stubGlobal("fetch", mockFetch);

    await githubStore.fetchStars();

    const state = get(githubStore);
    expect(state.stars).toBe("123");
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining("khun-phaen-tracker"),
    );
  });

  it("should handle fetch errors gracefully", async () => {
    githubStore.updateCache("10");

    const mockFetch = vi.fn().mockRejectedValue(new Error("API Down"));
    vi.stubGlobal("fetch", mockFetch);

    await githubStore.fetchStars();

    const state = get(githubStore);
    // Should keep old value on error
    expect(state.stars).toBe("10");
  });
});
