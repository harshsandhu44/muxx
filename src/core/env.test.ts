import { test } from "node:test";
import assert from "node:assert/strict";
import * as os from "os";
import * as path from "path";
import { expandHome, isInsideTmux } from "./env.js";

test("expandHome: '~' alone expands to home directory", () => {
  assert.equal(expandHome("~"), os.homedir());
});

test("expandHome: '~/foo' expands to home + foo", () => {
  assert.equal(expandHome("~/foo"), path.join(os.homedir(), "foo"));
});

test("expandHome: '~/a/b/c' expands nested path", () => {
  assert.equal(expandHome("~/a/b/c"), path.join(os.homedir(), "a/b/c"));
});

test("expandHome: absolute path is returned unchanged", () => {
  assert.equal(expandHome("/usr/local/bin"), "/usr/local/bin");
});

test("expandHome: relative path without tilde is returned unchanged", () => {
  assert.equal(expandHome("some/relative/path"), "some/relative/path");
});

test("expandHome: string starting with ~word is returned unchanged", () => {
  // '~user' is not home expansion — only '~' or '~/'
  assert.equal(expandHome("~user/path"), "~user/path");
});

test("isInsideTmux: returns false when TMUX is unset", () => {
  const original = process.env.TMUX;
  delete process.env.TMUX;
  assert.equal(isInsideTmux(), false);
  if (original !== undefined) process.env.TMUX = original;
});

test("isInsideTmux: returns false when TMUX is empty string", () => {
  const original = process.env.TMUX;
  process.env.TMUX = "";
  assert.equal(isInsideTmux(), false);
  if (original !== undefined) process.env.TMUX = original;
  else delete process.env.TMUX;
});

test("isInsideTmux: returns true when TMUX is set to a non-empty string", () => {
  const original = process.env.TMUX;
  process.env.TMUX = "/tmp/tmux-1000/default,12345,0";
  assert.equal(isInsideTmux(), true);
  if (original !== undefined) process.env.TMUX = original;
  else delete process.env.TMUX;
});
