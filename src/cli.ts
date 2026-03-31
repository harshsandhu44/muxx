#!/usr/bin/env node

import { error } from "./lib/out.js";

const [, , command, ...rest] = process.argv;

function printHelp(): void {
  console.log(
    `muxx — minimal tmux session manager

Usage:
  muxx                                             Connect to a session in cwd
  muxx connect|c [dir] [--name <n>] [--no-attach] Connect to or create a session
                        [--cmd "<cmd>"]
  muxx list|ls [--json]                            List all tmux sessions
  muxx kill|k <name> [--force]                     Kill a session by name
  muxx current|cur                                 Print the current session name`
  );
}

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
    case "kill":
    case "k": {
      if (!rest[0] || rest[0].startsWith("-")) {
        console.error("usage: muxx kill <name> [--force]");
        process.exit(1);
      }
      const { kill } = await import("./commands/kill.js");
      await kill(rest[0], rest.slice(1));
      break;
    }
    case "current":
    case "cur": {
      const { current } = await import("./commands/current.js");
      await current();
      break;
    }
    case undefined: {
      // no subcommand: connect to current directory
      const { connect } = await import("./commands/connect.js");
      await connect([]);
      break;
    }
    case "--help":
    case "-h": {
      printHelp();
      break;
    }
    default: {
      console.error(`unknown command: ${command}\nRun muxx --help for usage.`);
      process.exit(1);
    }
  }
}

main().catch((err: unknown) => {
  error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});
