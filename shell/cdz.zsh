# cdz — fuzzy recursive cd, scoped to your current directory.
#
# Source this from your ~/.zshrc:
#   source /path/to/cdz.zsh
#
# Then just run:
#   i test         # cd to the best fuzzy match for "test" under $PWD
#   i ../foo       # scoped to ../ first, then fuzzy-searching "foo"
#   i te<TAB>      # see (and pick from) the top 5 matches before committing
#   i -2           # cd up 2 directories (../..), same idea as pushd -N
#
# Bind it to whatever name you like — "i" here to keep it a one-key reach.
# `cdz` itself only prints matching paths on stdout; it never touches your
# shell's directory on its own, which is why this wrapper exists.

i() {
  # `i` alone — same as plain `cd`: go home.
  if [ $# -eq 0 ]; then
    cd "$HOME" || return $?
    return 0
  fi

  # `i -N` — jump straight up N directories, no fuzzy search involved.
  if [ $# -eq 1 ] && [[ "$1" =~ ^-[0-9]+$ ]]; then
    local n=${1#-}
    local up="" j
    for ((j = 0; j < n; j++)); do up="../$up"; done
    cd "$up" || return $?
    return 0
  fi

  # If a tab-completion candidate (a real directory) was accepted, cd
  # straight there instead of re-running the fuzzy search on it.
  if [ $# -eq 1 ] && [ -d "$1" ]; then
    cd "$1" || return $?
    return 0
  fi

  local dest
  dest="$(command cdz "$@")"
  local rc=$?
  if [ $rc -eq 0 ] && [ -n "$dest" ]; then
    cd "$dest" || return $?
  fi
  return $rc
}

# `i <TAB>` shows the top fuzzy matches immediately, same spirit as cd's
# own tab-completion — no need to type anything first. `menu select=1`
# puts you straight into a cyclable menu (Tab/arrows to move through it)
# even when there's only one, exact match. Only the substring you've
# actually typed gets colored (cyan), everything else stays plain — done
# via zsh's own "=pattern=color" list-colors form (not hand-rolled ANSI,
# which zsh's lister escapes/shows as "^[[..." instead of rendering).
_i() {
  local -a values
  values=(${(f)"$(command cdz --complete -- "$PREFIX" 2>/dev/null)"})
  values=(${values:#})  # drop any blank lines — an empty candidate would
                         # otherwise get auto-accepted, wiping what you typed
  (( $#values )) || return 1

  zstyle ':completion:*:*:i:*' menu select=1
  if [[ -n $PREFIX ]]; then
    # Matches are fuzzy, so the typed characters rarely sit together as one
    # contiguous substring in the result — a plain "*(prefix)*" pattern
    # only lights up the few candidates where they happen to. Build a
    # pattern with one capture group per typed character instead, so each
    # one gets colored wherever it lands, in order, same idea as fzf's
    # match highlighting.
    local pat="(#b)(#i)*" colors="0" c
    for c in ${(s::)PREFIX}; do
      pat+="(${(b)c})*"
      colors+="=1;36"
    done
    zstyle ':completion:*:*:i:*' list-colors "=${pat}=${colors}"
  else
    zstyle -d ':completion:*:*:i:*' list-colors
  fi
  compadd -l -Q -U -a values
  # Candidates are fuzzy matches, not prefix-related to what you typed, so
  # they rarely share a common prefix with each other. Without this, zsh's
  # first Tab tries to insert that (empty) common prefix before showing a
  # menu — which looks like your typed text just got wiped, especially in
  # a big folder where the top matches are more loosely related. Forcing
  # menu mode here skips straight to showing/selecting candidates instead.
  compstate[insert]=menu
}
compdef _i i

# Vanilla zsh only lets Return *accept* the highlighted menu candidate; it
# takes a second Return to actually run the line. Rebinding Return inside
# the menu keymap to the real accept-line (the "." prefix bypasses any
# other widget wrapping it) makes one Enter both pick the suggestion and
# run it, like fzf-style pickers do.
zmodload zsh/complist 2>/dev/null
bindkey -M menuselect '^M' .accept-line
