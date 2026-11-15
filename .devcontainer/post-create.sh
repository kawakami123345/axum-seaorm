#!/usr/bin/env bash
set -euo pipefail

cd /workspaces/project

cache_owner="$(id -u):$(id -g)"
cache_directories=(
  /workspaces/project/target
  /workspaces/project/web/node_modules
  /home/vscode/.cargo/registry
  /home/vscode/.cargo/git
)

# Docker can create the parent of the Cargo cache mounts as root.
sudo chown -- "$cache_owner" /home/vscode/.cargo
for directory in "${cache_directories[@]}"; do
  # Avoid traversing populated caches on every rebuild. A UID/GID change
  # requires updating their contents as well as the volume root.
  if [[ "$(stat -c '%u:%g' -- "$directory")" != "$cache_owner" ]]; then
    sudo chown -R -- "$cache_owner" "$directory"
  fi
done

mise trust
mise install
mise run setup
