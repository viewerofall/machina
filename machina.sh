# machina shell wrapper — `cd` to wherever you were when you quit
# Source from ~/.zshrc:    source ~/machina/machina.sh
# Then run `mc` instead of `machina`.

mc() {
    local tmp
    tmp="$(mktemp -t "machina.cwd.XXXXXX")"
    MACHINA_CWD_FILE="$tmp" command machina "$@"
    local cwd
    if [[ -s "$tmp" ]]; then
        cwd="$(cat -- "$tmp")"
        if [[ -n "$cwd" && "$cwd" != "$PWD" ]]; then
            builtin cd -- "$cwd"
        fi
    fi
    rm -f -- "$tmp"
}
