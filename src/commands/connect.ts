// muxx connect [target] [--name <name>] [--no-attach] [--cmd "<command>"]
//
// --cmd runs once, only when a brand-new session is created.
// It is sent verbatim to the shell via tmux send-keys — no escaping is done.
// Re-connecting to an existing session will NOT re-run the command.

import { hasTmux } from "../core/which.js";
import { resolveDir, isInsideTmux } from "../core/env.js";
import { resolveSessionName } from "../core/session-name.js";
import { hasSession, createSession, sendKeys, attachSession, switchClient } from "../core/tmux.js";
import { loadConfig, resolveProject } from "../core/config.js";
import { success, info, error } from "../lib/out.js";

function parseArgs(args: string[]): { target?: string; name?: string; noAttach: boolean; cmd?: string } {
  let target: string | undefined;
  let name: string | undefined;
  let noAttach = false;
  let cmd: string | undefined;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--name" && args[i + 1]) {
      name = args[++i];
    } else if (args[i] === "--no-attach") {
      noAttach = true;
    } else if (args[i] === "--cmd" && args[i + 1]) {
      cmd = args[++i];
    } else if (!args[i].startsWith("--")) {
      target = args[i];
    }
  }

  return { target, name, noAttach, cmd };
}

export async function connect(args: string[] = []): Promise<void> {
  if (!hasTmux()) {
    error("tmux not found in PATH");
    process.exit(1);
  }

  const { target, name: nameOverride, noAttach, cmd: cmdFlag } = parseArgs(args);

  const config = loadConfig();
  const project = target ? resolveProject(config, target) : undefined;
  const dir = resolveDir(project ? project.cwd : target);
  const sessionName = resolveSessionName(dir, nameOverride);

  // --cmd takes precedence over config startup
  const startupCmd = cmdFlag ?? project?.startup;

  const existed = hasSession(sessionName);

  if (!existed) {
    const ok = createSession(sessionName, dir);
    if (!ok) {
      error(`failed to create session: ${sessionName}`);
      process.exit(1);
    }
    success(`created: ${sessionName}`);
    if (startupCmd) {
      sendKeys(sessionName, startupCmd);
    }
  } else {
    info(`reused: ${sessionName}`);
  }

  if (noAttach) return;

  if (isInsideTmux()) {
    const ok = switchClient(sessionName);
    if (!ok) {
      error(`failed to switch to session: ${sessionName}`);
      process.exit(1);
    }
  } else {
    const ok = attachSession(sessionName);
    if (!ok) {
      error(`failed to attach to session: ${sessionName}`);
      process.exit(1);
    }
  }
}
