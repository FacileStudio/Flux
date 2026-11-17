t() {
    if [ $# -gt 0 ]; then
        tmux "$@"
        return
    fi
    if tmux has-session 2>/dev/null; then
        tmux attach
        return
    fi
    tmux new-session -d -s main 2>/dev/null
    tmux attach
}

hc() {
    if [ -n "${HISTFILE:-}" ]; then
        : >| "$HISTFILE" 2>/dev/null || true
        fc -p "$HISTFILE"
    else
        fc -p
    fi
}

vis() {
    if [ $# -eq 1 ] && [ -d "$1" ]; then
        (cd "$1" && command vis)
    else
        command vis "$@"
    fi
}

gcs() {
    if [ -z "${1:-}" ]; then
        echo "Usage: gcs <owner/repo>" >&2
        return 1
    fi
    git clone "git@github.com:$1.git"
}
