// muxx kill <name> — kill a session by name

import { currentSession, hasSession, killSession } from "../core/tmux.js";

export async function kill(name: string, args: string[] = []): Promise<void> {
  const force = args.includes("--force");

  if (!hasSession(name)) {
    console.error(`session not found: ${name}`);
    process.exit(1);
  }

  const current = currentSession();
  if (!force && current === name) {
    console.error(
      `refusing to kill current session '${name}' (use --force to override)`
    );
    process.exit(1);
  }

  const ok = killSession(name);
  if (!ok) {
    console.error(`failed to kill session: ${name}`);
    process.exit(1);
  }

  console.log(`killed: ${name}`);
}
