#!/usr/bin/env bash
# Devcontainer post-create setup. Kept out of devcontainer.json so each step
# can be commented and the JSON stays a one-liner (same approach as
# Archont561/qgis-rs).
set -euo pipefail

pixi --version

# Android tooling is deliberately installed outside Pixi: Google distributes the
# SDK/NDK as platform SDKs, not as part of the repository's conda toolchain.
# Keep the versions here explicit so a rebuilt container is reproducible and so
# `tauri android build` works without Android Studio in the container.
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-/opt/android-sdk}"
ANDROID_HOME="${ANDROID_HOME:-$ANDROID_SDK_ROOT}"
ANDROID_API_LEVEL="${ANDROID_API_LEVEL:-35}"
ANDROID_BUILD_TOOLS="${ANDROID_BUILD_TOOLS:-35.0.0}"
ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION:-27.2.12479018}"
ANDROID_CMDLINE_TOOLS_VERSION="${ANDROID_CMDLINE_TOOLS_VERSION:-13114758}"

export ANDROID_HOME ANDROID_SDK_ROOT

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Android SDK setup requires the dev container to run setup.sh as root" >&2
  exit 1
fi

apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  ca-certificates \
  fontconfig \
  fonts-dejavu \
  openjdk-17-jdk-headless \
  unzip \
  wget \
  xvfb

mkdir -p "$ANDROID_SDK_ROOT/cmdline-tools"
if [[ ! -x "$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager" ]]; then
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT
  wget -q "https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_CMDLINE_TOOLS_VERSION}_latest.zip" \
    -O "$tmpdir/cmdline-tools.zip"
  rm -rf "$tmpdir/cmdline-tools" "$ANDROID_SDK_ROOT/cmdline-tools/latest"
  unzip -q "$tmpdir/cmdline-tools.zip" -d "$tmpdir"
  mv "$tmpdir/cmdline-tools" "$ANDROID_SDK_ROOT/cmdline-tools/latest"
fi

export PATH="$ANDROID_SDK_ROOT/platform-tools:$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:$PATH"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-17-openjdk-amd64}"

yes | sdkmanager --licenses >/dev/null || true
sdkmanager \
  "platform-tools" \
  "platforms;android-${ANDROID_API_LEVEL}" \
  "build-tools;${ANDROID_BUILD_TOOLS}" \
  "ndk;${ANDROID_NDK_VERSION}"

cat >/etc/profile.d/castellan-android.sh <<EOF
export JAVA_HOME="$JAVA_HOME"
export ANDROID_HOME="$ANDROID_HOME"
export ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"
export PATH="$ANDROID_SDK_ROOT/platform-tools:$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:\$PATH"
if [ -d "$ANDROID_SDK_ROOT/ndk" ]; then
  export NDK_HOME="\$(find "$ANDROID_SDK_ROOT/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)"
fi
EOF

# The Pixi Rust package does not include Android standard libraries. If a
# rustup-managed toolchain is available, install the mobile targets; otherwise
# leave the explicit prerequisite visible rather than silently changing the
# repository's pinned Pixi toolchain.
if command -v rustup >/dev/null 2>&1; then
  rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
else
  echo "Note: rustup is not installed; install Rust Android targets before tauri android dev/build."
fi

# Baseline CLI tooling that the pixi image does not ship with.
pixi global install git gh

# A C toolchain is needed to build the Rust crates. The conda compiler
# binaries are prefixed with the target triple, so expose the unprefixed
# names (cc/gcc/ar) that build scripts expect.
pixi global install \
  --expose cc \
  --expose gcc \
  --expose ar=x86_64-conda-linux-gnu-ar \
  c-compiler

# Project environments, pinned to the lockfile for reproducibility.
pixi install --locked --all

# Workspace JavaScript dependencies (bun install --frozen-lockfile at the root).
pixi run bun install --frozen-lockfile

# ---------------------------------------------------------------------------
# opencode: the agent CLI. A developer tool, so it is installed here rather than
# as a pixi task — `pixi.toml` describes what the repository is built, tested and
# released with, and a 185 MB agent binary is none of those. `bun` comes from the
# default environment, which `pixi install --locked --all` above already materialised.
#
# bun's global bin dir (`bun pm bin -g`, which follows $BUN_INSTALL rather than a
# hardcoded $HOME/.bun/bin) is on nobody's PATH, so link the binary into a directory
# that already is: /usr/local/bin first (the container is root), ~/.local/bin second.
# The install itself still succeeded if neither is writable, so report where it
# landed instead of failing the container over a $PATH detail.
# ---------------------------------------------------------------------------
pixi run bun add -g opencode-ai@latest

BUN_BIN="$(pixi run bun pm bin -g | tail -n1)"
OPENCODE="$BUN_BIN/opencode"
if ln -sfn "$OPENCODE" /usr/local/bin/opencode 2> /dev/null; then
  OPENCODE=opencode
elif mkdir -p "$HOME/.local/bin" 2> /dev/null && ln -sfn "$OPENCODE" "$HOME/.local/bin/opencode" 2> /dev/null; then
  OPENCODE="$HOME/.local/bin/opencode"
else
  echo "opencode installed to $BUN_BIN, which is not on PATH: neither /usr/local/bin nor ~/.local/bin is writable"
fi

# The model catalogue is a separate cache (~/.cache/opencode/models.json, rebuilt from
# models.dev) and a binary upgrade does not invalidate it, so refreshing is its own step —
# otherwise a new release lists stale models.
"$BUN_BIN/opencode" models --refresh

cat <<EOF

────────────────────────────────────────────────────────────
opencode is ready. Start a session in this Codespace with:

  $OPENCODE -m opencode/big-pickle

────────────────────────────────────────────────────────────
EOF
