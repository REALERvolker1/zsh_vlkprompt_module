#!/bin/sh
set -eu

export ZDOTDIR="${0%/*}/gitignored/zdotdir"

#shellcheck disable=2016
_check_for() {
    if test "$1" "$2"; then
        printf 'Found `%s`\n' "$2"
    else
        printf 'Could not find `%s`\n' "$2"
        exit 1
    fi
}

cargo build

_check_for -d "$ZDOTDIR"
_check_for -r "$ZDOTDIR/.zshrc"

zsh
