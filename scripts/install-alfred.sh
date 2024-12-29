#!/bin/sh
# TODO: check if really need to be sudo
# TODO: add support for non-APT package managements
LATEST_VERSION=v0.1.0
ARCH=$(arch)
OS=$(uname)

echo "Installing prerequisites..."
sudo apt-get install -y tar

echo "Downloading Alfred core from github (${LATEST_VERSION})"

curl --output /tmp/alfred-rs.tar.gz "https://github.com/PaoloPana/alfred-rs/releases/download/${LATEST_VERSION}/alfred-rs_${ARCH}.tar.gz"

tar -xvzf /tmp/alfred-rs.tar.gz -C /usr/share/alfred
# TODO: ask to create default config files
# TODO: ask to add as service