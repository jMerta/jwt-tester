import { describe, it, expect } from "vitest";
import {
  isProjectIdValid,
  normalizeProjectId,
  pickProjectId,
} from "../projectSelection.js";

describe("projectSelection", () => {
  const projects = [{ id: "alpha" }, { id: "beta" }];

  it("validates project ids against known projects", () => {
    expect(isProjectIdValid("alpha", projects)).toBe(true);
    expect(isProjectIdValid("missing", projects)).toBe(false);
    expect(isProjectIdValid("", projects)).toBe(false);
  });

  it("normalizes unknown ids to null", () => {
    expect(normalizeProjectId("beta", projects)).toBe("beta");
    expect(normalizeProjectId("missing", projects)).toBe(null);
  });

  it("picks project id in priority order", () => {
    const result = pickProjectId({
      preferredId: "beta",
      selectedId: "alpha",
      defaultId: null,
      lastId: null,
      projects,
    });
    expect(result).toBe("beta");
  });

  it("falls back to default or last when preferred/selected invalid", () => {
    const result = pickProjectId({
      preferredId: "missing",
      selectedId: "missing",
      defaultId: "alpha",
      lastId: "beta",
      projects,
    });
    expect(result).toBe("alpha");
  });

  it("falls back to first project when no ids match", () => {
    const result = pickProjectId({
      preferredId: null,
      selectedId: null,
      defaultId: null,
      lastId: null,
      projects,
    });
    expect(result).toBe("alpha");
  });
});
