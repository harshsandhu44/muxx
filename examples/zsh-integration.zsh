# muxx zsh integration
# Add to ~/.zshrc

# mx [dir-or-alias]
# Connect to a muxx session for the given directory or config alias.
# Defaults to the current directory when called with no argument.
function mx() {
  muxx connect "${1:-.}"
}

# mxp — interactive session picker (requires fzf)
# Lists existing tmux sessions and connects to the selected one.
# Falls back to a plain message if fzf is not installed.
function mxp() {
  if ! command -v fzf &>/dev/null; then
    echo "mxp requires fzf (https://github.com/junegunn/fzf)"
    return 1
  fi

  local session
  session=$(muxx list --json | python3 -c "
import json, sys
sessions = json.load(sys.stdin)
for s in sessions:
    print(s['name'])
" | fzf --prompt="session> " --height=10 --reverse)

  [[ -n "$session" ]] && muxx connect "$session"
}
