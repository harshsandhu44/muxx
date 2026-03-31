import { run } from "./run.js";

/** Returns true if the given binary is found on PATH. */
export function isBinaryAvailable(bin: string): boolean {
  const result = run("which", [bin]);
  return result.exitCode === 0;
}

export const hasTmux = () => isBinaryAvailable("tmux");
