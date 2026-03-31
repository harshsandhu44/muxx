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
    case "connect": {
      const { connect } = await import("./commands/connect.js");
      await connect(rest[0]);
      break;
    }
    case "kill": {
      if (!rest[0]) {
        console.error("Usage: muxx kill <name>");
        process.exit(1);
      }
      const { kill } = await import("./commands/kill.js");
      await kill(rest[0]);
      break;
    }
    case "current": {
      const { current } = await import("./commands/current.js");
      await current();
      break;
    }
    default: {
      console.log(
        `muxx — minimal tmux session CLI

Usage:
  muxx list|ls           List all tmux sessions
  muxx connect [target]  Attach to a session (defaults to most recent)
  muxx kill <name>       Kill a session by name
  muxx current           Print the currently attached session`
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
