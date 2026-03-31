import { test } from "node:test";
import assert from "node:assert/strict";
import { resolveProject } from "./config.js";
import type { MuxxConfig } from "./config.js";

const baseConfig: MuxxConfig = {
  projects: {
    myapp: { cwd: "/home/user/myapp" },
    api: { cwd: "/home/user/api", startup: "npm run dev" },
  },
};

test("resolveProject: returns config for existing key", () => {
  const result = resolveProject(baseConfig, "myapp");
  assert.deepEqual(result, { cwd: "/home/user/myapp" });
});

test("resolveProject: returns config with startup command", () => {
  const result = resolveProject(baseConfig, "api");
  assert.deepEqual(result, { cwd: "/home/user/api", startup: "npm run dev" });
});

test("resolveProject: returns undefined for missing key", () => {
  const result = resolveProject(baseConfig, "nonexistent");
  assert.equal(result, undefined);
});

test("resolveProject: returns undefined on empty projects map", () => {
  const result = resolveProject({ projects: {} }, "anything");
  assert.equal(result, undefined);
});
