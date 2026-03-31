// muxx current — print the name of the currently attached session

import { isInsideTmux } from "../core/env.js";
import { currentSession } from "../core/tmux.js";

export async function current(): Promise<void> {
  if (!isInsideTmux()) {
    console.error("not inside a tmux session");
    process.exit(1);
  }

  const name = currentSession();
  if (!name) {
    console.error("could not determine current session");
    process.exit(1);
  }

  console.log(name);
}
