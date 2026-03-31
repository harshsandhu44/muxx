import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expandHome } from "./env.js";

export interface ProjectConfig {
  cwd: string;
}

export interface MuxxConfig {
  projects: Record<string, ProjectConfig>;
}

const CONFIG_PATH = expandHome("~/.config/muxx/config.json");

export function loadConfig(): MuxxConfig {
  try {
    const raw = readFileSync(CONFIG_PATH, "utf-8");
    return JSON.parse(raw) as MuxxConfig;
  } catch (err: unknown) {
    if (isNodeError(err) && err.code === "ENOENT") {
      return { projects: {} };
    }
    const msg = err instanceof SyntaxError ? `invalid JSON in ${CONFIG_PATH}: ${err.message}` : `failed to read config: ${CONFIG_PATH}`;
    console.error(msg);
    process.exit(1);
  }
}

export function resolveProject(config: MuxxConfig, key: string): ProjectConfig | undefined {
  return config.projects[key];
}

function isNodeError(err: unknown): err is NodeJS.ErrnoException {
  return err instanceof Error && "code" in err;
}
