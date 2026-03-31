export interface TmuxSession {
  name: string;
  windows: number;
  created: Date;
  attached: boolean;
}

export interface CommandContext {
  args: string[];
  flags: Record<string, string | boolean>;
}
