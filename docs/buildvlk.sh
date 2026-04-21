#!/usr/bin/env bash
# My script for building zsh from source
# Remote: git://git.code.sf.net/p/zsh/code
set -euo pipefail
IFS=$'\t\n'

declare -a zflags=()

case "${1-}" in
  -d|--debug)
  export CFLAGS='-march=native'
  zflags+=(--enable-zsh-valgrind --enable-zsh-secure-free --enable-zsh-debug)
  ;;
  -r|--release)
  export CFLAGS='-O3 -march=native'
  ;;
  *)
  printf '%s\n' 'Error, available args:' \
    '-d, --debug    configure with debug mode' \
    '-r, --release  configure with release mode'
    exit 1
    ;;
esac




cwd="$(realpath "${0%/*}")"
cd "$cwd"

make clean-recursive

"$cwd"/configure \
  --enable-custom-patchlevel="buildvlk" \
  --enable-pcre --enable-cap --enable-gdbm \
  --enable-multibyte \
  --prefix="$cwd/prefix" \
  "${zflags[@]}"
make
make install
