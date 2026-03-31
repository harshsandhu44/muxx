// muxx kill <name> — kill a session by name

import { hasTmux } from "../core/which.js";
import { currentSession, hasSession, killSession } from "../core/tmux.js";
import { success, error } from "../lib/out.js";

export async function kill(name: string, args: string[] = []): Promise<void> {
  if (!hasTmux()) {
    error("tmux not found in PATH");
    process.exit(1);
  }

  const force = args.includes("--force");

  if (!hasSession(name)) {
    error(`session not found: ${name}`);
    process.exit(1);
  }

  const current = currentSession();
  if (!force && current === name) {
    error(`refusing to kill current session '${name}' (use --force to override)`);
    process.exit(1);
  }

  const ok = killSession(name);
  if (!ok) {
    error(`failed to kill session: ${name}`);
    process.exit(1);
  }

  success(`killed: ${name}`);
}
