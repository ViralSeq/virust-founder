#!/usr/bin/env bash
set -euo pipefail

ENV_NAME="founder"
ENV_FILE="founder.conda.yml"

# Install conda (Miniforge) if missing
if ! command -v conda >/dev/null 2>&1; then
  echo "Conda not found."
  read -r -p "Press Y to install Conda (Miniforge), or any other key to abort: " REPLY
  if [ "$REPLY" != "Y" ] && [ "$REPLY" != "y" ]; then
    echo "Conda required to run.  Exiting."
    exit 1
  fi
  echo "Installing Miniforge..."
  OS="$(uname -s)"
  ARCH="$(uname -m)"
  case "${OS}_${ARCH}" in
    Darwin_arm64) URL="https://github.com/conda-forge/miniforge/releases/latest/download/Miniforge3-MacOSX-arm64.sh" ;;
    Darwin_x86_64) URL="https://github.com/conda-forge/miniforge/releases/latest/download/Miniforge3-MacOSX-x86_64.sh" ;;
    Linux_x86_64) URL="https://github.com/conda-forge/miniforge/releases/latest/download/Miniforge3-Linux-x86_64.sh" ;;
    *) echo "Unsupported platform: ${OS}_${ARCH}"; exit 1 ;;
  esac
  INSTALLER="/tmp/miniforge.sh"
  curl -L "$URL" -o "$INSTALLER"
  bash "$INSTALLER" -b -p "$HOME/miniforge3"
  export PATH="$HOME/miniforge3/bin:$PATH"
fi

# Create or update the conda env
conda env update -n "$ENV_NAME" -f "$ENV_FILE" --prune \
  || conda env create -n "$ENV_NAME" -f "$ENV_FILE"

# Ensure RubyGems native extensions are built for this env's Ruby
# (prevents: "Ignoring date-... because its extensions are not built")
conda run -n "$ENV_NAME" ruby -S gem pristine date --version 3.3.4 >/dev/null 2>&1 || true

# Clone hyphy analyses if missing
if [ ! -d "hyphy-analyses" ]; then
  git clone https://github.com/veg/hyphy-analyses
fi

# Install viral_seq using the env's Ruby (avoid calling system Ruby/gem)
conda run -n "$ENV_NAME" ruby -S gem install viral_seq -v 1.10.0

echo
echo "Done."
echo "Run with:"
echo "  conda run -n $ENV_NAME cargo run --release -- <args>"
