// Session name sanitization utilities

/**
 * Convert a raw string or path into a safe tmux session name.
 *
 * Rules applied in order:
 *   1. Extract the last path segment (basename) if the input looks like a path
 *   2. Lowercase
 *   3. Trim whitespace
 *   4. Replace spaces with hyphens
 *   5. Replace invalid characters (anything not a-z, 0-9, hyphen, underscore) with hyphens
 *   6. Collapse repeated hyphens
 *   7. Remove leading/trailing hyphens
 *
 * Examples:
 *   "/Users/harsh/Code/my-project"  → "my-project"
 *   "My Cool App"                   → "my-cool-app"
 *   "foo//bar"                      → "foo-bar"
 *   "  hello world  "               → "hello-world"
 *   "foo---bar"                     → "foo-bar"
 *   "/tmp/"                         → "tmp"
 *   "UPPER_CASE"                    → "upper_case"
 */
export function sanitizeSessionName(input: string): string {
  // Use last non-empty path segment for anything that looks like a path
  const segments = input.split("/").filter(Boolean);
  const base = segments.length > 1 ? segments[segments.length - 1] : input;

  return base
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "-")
    .replace(/[^a-z0-9_-]/g, "-")
    .replace(/-{2,}/g, "-")
    .replace(/^-+|-+$/g, "");
}

/**
 * Resolve the effective session name, letting an explicit override win.
 * Falls back to sanitizing the raw input when no override is provided.
 */
export function resolveSessionName(raw: string, override?: string): string {
  if (override !== undefined && override.trim().length > 0) {
    return sanitizeSessionName(override);
  }
  return sanitizeSessionName(raw);
}
