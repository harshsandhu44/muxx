#!/usr/bin/env node

const [, , command, ...rest] = process.argv;

async function main(): Promise<void> {
  switch (command) {
    case "list":
    case "ls": {
      const { list } = await import("./commands/list.js");
      await list(rest);
      break;
    }
    case "connect":
    case "c": {
      const { connect } = await import("./commands/connect.js");
      await connect(rest);
      break;
    }
    case "kill": {
      if (!rest[0]) {
        console.error("Usage: muxx kill <name>");
        process.exit(1);
      }
      const { kill } = await import("./commands/kill.js");
      await kill(rest[0], rest.slice(1));
      break;
    }
    case "current": {
      const { current } = await import("./commands/current.js");
      await current();
      break;
    }
    default: {
      console.log(
        `muxx — minimal tmux session manager

Usage:
  muxx list|ls [--json]                         List all tmux sessions
  muxx connect [dir] [--name <n>] [--no-attach] Connect to a session (default: cwd)
  muxx kill <name> [--force]                    Kill a session by name
  muxx current                                  Print the current session`
      );
      if (command !== undefined && command !== "--help" && command !== "-h") {
        console.error(`\nUnknown command: ${command}`);
        process.exit(1);
      }
    }
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
