// @vitest-environment jsdom
import { render, screen } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import AccessDenied from "./AccessDenied.svelte";
import { _ } from "svelte-i18n";

// Mock svelte-i18n
vi.mock("svelte-i18n", () => {
  return {
    _: {
      subscribe: (fn: any) => {
        fn((key: string) => key);
        return () => {};
      },
    },
  };
});

// Mock svelte-app navigation and paths
vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));
vi.mock("$app/paths", () => ({
  base: "",
}));

describe("AccessDenied", () => {
  it("renders the access denied message", () => {
    render(AccessDenied);
    // Default text from mock should be the key itself since my mock returns the key
    expect(screen.getByText("page__no_access")).toBeTruthy();
    expect(screen.getByText("page__back_to_dashboard")).toBeTruthy();
  });
});
