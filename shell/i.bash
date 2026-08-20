# i — fuzzy recursive cd, scoped to your current directory.
#
# Source this from your ~/.bashrc:
#   source /path/to/i.bash
#
# Then just run:
#   i test         # cd to the best fuzzy match for "test" under $PWD
#   i ../foo       # scoped to ../ first, then fuzzy-searching "foo"
#   i te<TAB>      # cycle through the top matches before committing
#   i -2           # cd up 2 directories (../..), same idea as pushd -N
#
# The underlying `i` binary only prints matching paths on stdout; it never
# touches your shell's directory on its own, which is why this wrapper
# exists.

i() {
  if [ $# -eq 0 ]; then
    cd "$HOME" || return $?
    return 0
  fi

  case "$1" in
    --help|-h|--version|-V|--config) command i "$@"; return $? ;;
  esac

  if [ $# -eq 1 ] && [[ "$1" =~ ^-[0-9]+$ ]]; then
    local n=${1#-}
    local up="" j
    for ((j = 0; j < n; j++)); do up="../$up"; done
    cd "$up" || return $?
    return 0
  fi

  if [ $# -eq 1 ] && [ -d "$1" ]; then
    cd "$1" || return $?
    return 0
  fi

  local dest
  dest="$(command i "$@")"
  local rc=$?
  if [ $rc -eq 0 ] && [ -n "$dest" ]; then
    cd "$dest" || return $?
  fi
  return $rc
}

# `i te<TAB>` — offers the top fuzzy matches via bash's completion. No
# highlighting/menu-select like the zsh version (bash's completion system
# doesn't support either), just a plain candidate list.
_i() {
  local IFS=$'\n'
  COMPREPLY=($(command i --complete -- "${COMP_WORDS[COMP_CWORD]}" 2>/dev/null))
}
complete -F _i i
