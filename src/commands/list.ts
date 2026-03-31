// muxx list — show all tmux sessions

import { hasTmux } from "../core/which.js";
import { listSessions } from "../core/tmux.js";

export async function list(args: string[] = []): Promise<void> {
  const json = args.includes("--json");

  if (!hasTmux()) {
    console.error("error: tmux is not installed or not in PATH");
    process.exit(1);
  }

  const sessions = listSessions();

  if (json) {
    console.log(JSON.stringify(sessions, null, 2));
    return;
  }

  if (sessions.length === 0) {
    console.log("no tmux sessions");
    return;
  }

  for (const s of sessions) {
    const state = s.attached ? "attached" : "detached";
    const wins = s.windows === 1 ? "1 window" : `${s.windows} windows`;
    console.log(`${s.name}  ${wins}  [${state}]`);
  }
}
