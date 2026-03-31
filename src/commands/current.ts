// muxx current — print the name of the currently attached session

import { isInsideTmux } from "../core/env.js";
import { run } from "../core/run.js";

export async function current(): Promise<void> {
  if (!isInsideTmux()) {
    console.error("not inside a tmux session");
    process.exit(1);
  }

  const { stdout, exitCode } = run("tmux", ["display-message", "-p", "#S"]);
  if (exitCode !== 0) {
    console.error("could not determine current session");
    process.exit(1);
  }

  process.stdout.write(stdout.trimEnd() + "\n");
}
