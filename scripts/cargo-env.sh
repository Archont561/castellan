#!/bin/sh
# Rust-lane router, called by the @castellan/rust turbo facade (crates/package.json)
# instead of cargo directly. Two destination environments:
#
#   shells   — whenever the local checkout has pixi's `shells` environment
#              materialized, cargo runs there. That env carries the conda-forge
#              gtk/webkit stack (feature gtk-shells), so `--workspace` lanes
#              compile the Tauri shells with no system packages at all —
#              the whole point of the two-environment split (decision-13).
#   ambient  — where `shells` is not installed, cargo runs against the
#              toolchain on PATH and gtk/webkit comes from the system
#              (CI's apt step, a contributor's dev packages), which is
#              exactly how every lane behaved before the split.
#
# CASTELLAN_CARGO_ENV overrides the detection: an env name forces routing,
# `off` forces the ambient branch.
set -eu
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
ENV_NAME=${CASTELLAN_CARGO_ENV:-auto}
case $ENV_NAME in
  off) exec cargo "$@" ;;
  auto)
    if [ -d "$ROOT/.pixi/envs/shells" ]; then
      ENV_NAME=shells
    else
      exec cargo "$@"
    fi ;;
esac
exec pixi run -e "$ENV_NAME" --manifest-path "$ROOT/pixi.toml" -- cargo "$@"
