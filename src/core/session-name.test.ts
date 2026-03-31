import { test } from "node:test";
import assert from "node:assert/strict";
import { sanitizeSessionName, resolveSessionName } from "./session-name.js";

test("sanitizeSessionName: extracts basename from absolute path", () => {
  assert.equal(sanitizeSessionName("/Users/harsh/Code/my-project"), "my-project");
});

test("sanitizeSessionName: extracts basename from path with trailing slash", () => {
  assert.equal(sanitizeSessionName("/tmp/"), "tmp");
});

test("sanitizeSessionName: lowercases input", () => {
  assert.equal(sanitizeSessionName("MyProject"), "myproject");
});

test("sanitizeSessionName: preserves underscores", () => {
  assert.equal(sanitizeSessionName("UPPER_CASE"), "upper_case");
});

test("sanitizeSessionName: replaces spaces with hyphens", () => {
  assert.equal(sanitizeSessionName("My Cool App"), "my-cool-app");
});

test("sanitizeSessionName: trims leading and trailing whitespace", () => {
  assert.equal(sanitizeSessionName("  hello world  "), "hello-world");
});

test("sanitizeSessionName: replaces invalid characters with hyphens", () => {
  assert.equal(sanitizeSessionName("foo@bar!baz"), "foo-bar-baz");
});

test("sanitizeSessionName: collapses repeated hyphens", () => {
  assert.equal(sanitizeSessionName("foo---bar"), "foo-bar");
});

test("sanitizeSessionName: removes leading and trailing hyphens", () => {
  assert.equal(sanitizeSessionName("-foo-"), "foo");
});

test("sanitizeSessionName: single-segment input is not treated as path", () => {
  assert.equal(sanitizeSessionName("myapp"), "myapp");
});

test("sanitizeSessionName: two-segment path extracts last segment", () => {
  assert.equal(sanitizeSessionName("Code/my-project"), "my-project");
});

test("resolveSessionName: sanitizes raw input when no override", () => {
  assert.equal(resolveSessionName("/home/user/my app"), "my-app");
});

test("resolveSessionName: uses override when provided", () => {
  assert.equal(resolveSessionName("/home/user/my app", "Custom Name"), "custom-name");
});

test("resolveSessionName: ignores blank override string", () => {
  assert.equal(resolveSessionName("/home/user/myapp", "   "), "myapp");
});

test("resolveSessionName: override with no raw path still sanitizes", () => {
  assert.equal(resolveSessionName("ignored", "My Override"), "my-override");
});
