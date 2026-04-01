// Core tmux interaction layer — all tmux exec calls go through here

import { run, runInteractive } from "./run.js";
import type { TmuxSession } from "../types/index.js";

// tmux list-sessions -F "#{session_name}:#{session_windows}:#{session_attached}:#{session_created}"
function parseSessions(raw: string): TmuxSession[] {
  return raw
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => {
      const [name, windows, attached, created] = line.split(":");
      return {
        name,
        windows: parseInt(windows, 10),
        attached: attached === "1",
        created: new Date(parseInt(created, 10) * 1000),
      };
    });
}

export function listSessions(): TmuxSession[] {
  const result = run("tmux", ["list-sessions", "-F", "#{session_name}:#{session_windows}:#{session_attached}:#{session_created}"]);
  if (result.exitCode !== 0) return [];
  return parseSessions(result.stdout);
}

export function hasSession(name: string): boolean {
  const result = run("tmux", ["has-session", "-t", name]);
  return result.exitCode === 0;
}

export function createSession(name: string, cwd: string): boolean {
  const result = run("tmux", ["new-session", "-d", "-s", name, "-c", cwd]);
  return result.exitCode === 0;
}

export function attachSession(name: string): boolean {
  return runInteractive("tmux", ["attach-session", "-t", name]) === 0;
}

export function switchClient(name: string): boolean {
  return runInteractive("tmux", ["switch-client", "-t", name]) === 0;
}

// Sends keystrokes to the first pane of a session, followed by Enter.
// The command string is passed as-is to the shell running in that pane —
// no escaping is performed by muxx. Runs only on new session creation.
export function sendKeys(session: string, cmd: string): boolean {
  const result = run("tmux", ["send-keys", "-t", `${session}:`, cmd, "Enter"]);
  return result.exitCode === 0;
}

export function killSession(name: string): boolean {
  const result = run("tmux", ["kill-session", "-t", name]);
  return result.exitCode === 0;
}

export function currentSession(): string | null {
  const result = run("tmux", ["display-message", "-p", "#{session_name}"]);
  if (result.exitCode !== 0) return null;
  const name = result.stdout.trim();
  return name.length > 0 ? name : null;
}
