// muxx connect [target] [--name <name>] [--no-attach]

import { hasTmux } from "../core/which.js";
import { resolveDir, isInsideTmux } from "../core/env.js";
import { resolveSessionName } from "../core/session-name.js";
import { hasSession, createSession, attachSession, switchClient } from "../core/tmux.js";

function parseArgs(args: string[]): { target?: string; name?: string; noAttach: boolean } {
  let target: string | undefined;
  let name: string | undefined;
  let noAttach = false;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--name" && args[i + 1]) {
      name = args[++i];
    } else if (args[i] === "--no-attach") {
      noAttach = true;
    } else if (!args[i].startsWith("--")) {
      target = args[i];
    }
  }

  return { target, name, noAttach };
}

export async function connect(args: string[] = []): Promise<void> {
  if (!hasTmux()) {
    console.error("tmux is not available");
    process.exit(1);
  }

  const { target, name: nameOverride, noAttach } = parseArgs(args);

  const dir = resolveDir(target);
  const sessionName = resolveSessionName(dir, nameOverride);

  const existed = hasSession(sessionName);

  if (!existed) {
    const ok = createSession(sessionName, dir);
    if (!ok) {
      console.error(`failed to create session: ${sessionName}`);
      process.exit(1);
    }
    console.log(`created: ${sessionName}`);
  }

  if (noAttach) return;

  if (isInsideTmux()) {
    const ok = switchClient(sessionName);
    if (!ok) {
      console.error(`failed to switch to session: ${sessionName}`);
      process.exit(1);
    }
  } else {
    const ok = attachSession(sessionName);
    if (!ok) {
      console.error(`failed to attach to session: ${sessionName}`);
      process.exit(1);
    }
  }
}
