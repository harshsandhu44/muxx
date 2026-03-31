import pc from "picocolors";

export function success(msg: string): void {
  console.log(`${pc.green("✓")} ${msg}`);
}

export function info(msg: string): void {
  console.log(`${pc.cyan("→")} ${msg}`);
}

export function warn(msg: string): void {
  console.error(`${pc.yellow("!")} ${msg}`);
}

export function error(msg: string): void {
  console.error(`${pc.red("✗")} ${msg}`);
}

export function hint(msg: string): void {
  console.log(pc.dim(`  ${msg}`));
}
