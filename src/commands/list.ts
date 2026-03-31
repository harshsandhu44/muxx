// muxx list — show all tmux sessions

import pc from "picocolors";
import { hasTmux } from "../core/which.js";
import { listSessions } from "../core/tmux.js";
import { error } from "../lib/out.js";

export async function list(args: string[] = []): Promise<void> {
  const json = args.includes("--json");

  if (!hasTmux()) {
    error("tmux not found in PATH");
    process.exit(1);
  }

  const sessions = listSessions();

  if (json) {
    console.log(JSON.stringify(sessions, null, 2));
    return;
  }

  if (sessions.length === 0) {
    console.log(pc.dim("no sessions"));
    return;
  }

  const nameWidth = Math.max(...sessions.map((s) => s.name.length));

  for (const s of sessions) {
    const name = s.name.padEnd(nameWidth);
    const wins = String(s.windows).padStart(2);
    const state = s.attached ? pc.green("attached") : pc.dim("detached");
    console.log(`${name}  ${wins}  ${state}`);
  }
}
