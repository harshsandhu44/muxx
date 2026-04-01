const bash = `\
# muxx bash completion
_muxx_completion() {
  local cur="\${COMP_WORDS[COMP_CWORD]}"
  local cmd="\${COMP_WORDS[1]}"

  if [[ $COMP_CWORD -eq 1 ]]; then
    COMPREPLY=(\$(compgen -W "connect c list ls kill k current cur completion --help -h" -- "$cur"))
    return 0
  fi

  case "$cmd" in
    connect|c)
      if [[ "$cur" == -* ]]; then
        COMPREPLY=(\$(compgen -W "--name --no-attach --cmd" -- "$cur"))
      else
        COMPREPLY=(\$(compgen -d -- "$cur"))
      fi
      ;;
    list|ls)
      COMPREPLY=(\$(compgen -W "--json" -- "$cur"))
      ;;
    kill|k)
      COMPREPLY=(\$(compgen -W "--force" -- "$cur"))
      ;;
    completion)
      COMPREPLY=(\$(compgen -W "bash zsh fish" -- "$cur"))
      ;;
  esac
}

complete -F _muxx_completion muxx
`;

const zsh = `\
#compdef muxx

_muxx() {
  local -a commands
  commands=(
    'connect:Connect to or create a tmux session'
    'c:Connect to or create a tmux session (alias)'
    'list:List all tmux sessions'
    'ls:List all tmux sessions (alias)'
    'kill:Kill a session by name'
    'k:Kill a session by name (alias)'
    'current:Print the current session name'
    'cur:Print the current session name (alias)'
    'completion:Print shell completion script'
  )

  if (( CURRENT == 2 )); then
    _describe 'command' commands
    return
  fi

  case "\$words[2]" in
    connect|c)
      _arguments \\
        '1:directory:_directories' \\
        '--name[override session name]:name' \\
        '--no-attach[create without attaching]' \\
        '--cmd[command to run on new session]:command'
      ;;
    list|ls)
      _arguments '--json[output as JSON]'
      ;;
    kill|k)
      _arguments '--force[force kill current session]'
      ;;
    completion)
      local -a shells
      shells=('bash' 'zsh' 'fish')
      _describe 'shell' shells
      ;;
  esac
}

compdef _muxx muxx
`;

const fish = `\
# muxx fish completion

complete -c muxx -n '__fish_seen_subcommand_from connect c' -F
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -f

complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'connect' -d 'Connect to or create a tmux session'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'c' -d 'Connect to or create a tmux session'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'list' -d 'List all tmux sessions'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'ls' -d 'List all tmux sessions'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'kill' -d 'Kill a session by name'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'k' -d 'Kill a session by name'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'current' -d 'Print the current session name'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'cur' -d 'Print the current session name'
complete -c muxx -n 'not __fish_seen_subcommand_from connect c list ls kill k current cur completion' -a 'completion' -d 'Print shell completion script'

complete -c muxx -n '__fish_seen_subcommand_from connect c' -l name -d 'Override session name'
complete -c muxx -n '__fish_seen_subcommand_from connect c' -l no-attach -d 'Create without attaching'
complete -c muxx -n '__fish_seen_subcommand_from connect c' -l cmd -d 'Command to run on new session'

complete -c muxx -n '__fish_seen_subcommand_from list ls' -l json -d 'Output as JSON'

complete -c muxx -n '__fish_seen_subcommand_from kill k' -l force -d 'Force kill current session'

complete -c muxx -n '__fish_seen_subcommand_from completion' -a 'bash zsh fish'
`;

const scripts: Record<string, string> = { bash, zsh, fish };

export function completion(args: string[]): void {
  const shell = args[0];

  if (!shell) {
    console.error("usage: muxx completion <bash|zsh|fish>");
    process.exit(1);
  }

  const script = scripts[shell];
  if (!script) {
    console.error(`unsupported shell: ${shell}\nSupported: bash, zsh, fish`);
    process.exit(1);
  }

  process.stdout.write(script);
}
