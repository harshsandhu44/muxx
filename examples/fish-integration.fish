# muxx fish integration
# Add to ~/.config/fish/config.fish, or drop this file in
# ~/.config/fish/conf.d/muxx.fish for auto-loading.

# mx [dir-or-alias]
# Connect to a muxx session for the given directory or config alias.
# Defaults to the current directory when called with no argument.
function mx
    muxx connect (or $argv[1] ".")
end

# mxk [session]
# Kill the named session interactively (requires fzf).
function mxk
    if not command -q fzf
        echo "mxk requires fzf (https://github.com/junegunn/fzf)"
        return 1
    end

    set session (muxx list --json | python3 -c "
import json, sys
sessions = json.load(sys.stdin)
for s in sessions:
    print(s['name'])
" | fzf --prompt="kill> " --height=10 --reverse)

    if test -n "$session"
        muxx kill $session
    end
end

# Shell completion (fish)
# Run once to generate the completions file:
#   muxx completion fish > ~/.config/fish/completions/muxx.fish
