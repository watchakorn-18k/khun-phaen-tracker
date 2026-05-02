import { describe, it, expect, vi, beforeEach } from "vitest";
import { isImage, processFilesForPreview, revokePreview } from "./file";

// Mock URL.createObjectURL and URL.revokeObjectURL
global.URL.createObjectURL = vi.fn((file) => `blob:${file.name}`);
global.URL.revokeObjectURL = vi.fn();

describe("file utility", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("isImage", () => {
    it("should return true for image files", () => {
      const file = new File([""], "test.png", { type: "image/png" });
      expect(isImage(file)).toBe(true);
    });

    it("should return false for non-image files", () => {
      const file = new File([""], "test.pdf", { type: "application/pdf" });
      expect(isImage(file)).toBe(false);
    });
  });

  describe("processFilesForPreview", () => {
    it("should generate previews for images but not for other files", () => {
      const files = [
        new File([""], "img.png", { type: "image/png" }),
        new File([""], "doc.pdf", { type: "application/pdf" }),
      ];
      
      const result = processFilesForPreview(files);
      
      expect(result).toHaveLength(2);
      expect(result[0].preview).toBe("blob:img.png");
      expect(result[1].preview).toBeUndefined();
      expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    });
  });

  describe("revokePreview", () => {
    it("should call revokeObjectURL for blob URLs", () => {
      revokePreview("blob:test");
      expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:test");
    });

    it("should not call revokeObjectURL for undefined", () => {
      revokePreview(undefined);
      expect(URL.revokeObjectURL).not.toHaveBeenCalled();
    });

    it("should not call revokeObjectURL for non-blob URLs", () => {
      revokePreview("http://example.com/test.png");
      expect(URL.revokeObjectURL).not.toHaveBeenCalled();
    });
  });
});
