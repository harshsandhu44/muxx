import * as fs from "fs";
import * as os from "os";
import * as path from "path";

/** Returns true when the current process is running inside a tmux session. */
export function isInsideTmux(): boolean {
  return typeof process.env.TMUX === "string" && process.env.TMUX.length > 0;
}

/**
 * Expand a leading `~` to the user's home directory.
 * Returns the input unchanged if it does not start with `~`.
 */
export function expandHome(input: string): string {
  if (input === "~") return os.homedir();
  if (input.startsWith("~/")) return path.join(os.homedir(), input.slice(2));
  return input;
}

/**
 * Resolve a target path to an absolute directory path.
 *
 * - If `target` is provided it is expanded and resolved; otherwise the
 *   current working directory is used.
 * - Throws if the resolved path does not exist or is not a directory.
 */
export function resolveDir(target?: string): string {
  const raw = target !== undefined && target.trim().length > 0
    ? target.trim()
    : process.cwd();

  const expanded = expandHome(raw);
  const resolved = path.resolve(expanded);

  assertDirectory(resolved);

  return resolved;
}

/**
 * Validate that `dirPath` exists and is a directory.
 * Throws a descriptive `Error` otherwise.
 */
export function assertDirectory(dirPath: string): void {
  let stat: fs.Stats;
  try {
    stat = fs.statSync(dirPath);
  } catch {
    throw new Error(`directory does not exist: ${dirPath}`);
  }

  if (!stat.isDirectory()) {
    throw new Error(`path is not a directory: ${dirPath}`);
  }
}
