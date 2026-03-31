import { spawnSync } from "child_process";

export interface RunResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export interface RunOptions {
  cwd?: string;
}

export function run(cmd: string, args: string[], opts: RunOptions = {}): RunResult {
  const result = spawnSync(cmd, args, {
    cwd: opts.cwd,
    encoding: "utf8",
    // Don't throw on non-zero exit — callers check exitCode
    stdio: ["ignore", "pipe", "pipe"],
  });

  return {
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    exitCode: result.status ?? 1,
  };
}
