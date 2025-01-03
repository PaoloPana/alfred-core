#!/bin/sh
LATEST_VERSION=v0.1.9
INSTALLATION_DIR="${HOME}/.local/share"
ALFRED_DIR="${INSTALLATION_DIR}/alfred"
ARCH=$(arch)
OS=$(uname)

create_default_config() {
    echo "Downloading default config files"
    BASE_LINK="https://github.com/PaoloPana/alfred-rs/raw/refs/tags/${LATEST_VERSION}"
    curl -L --output ${ALFRED_DIR}/config.toml "${BASE_LINK}/config.toml"
    curl -L --output ${ALFRED_DIR}/repositories.toml "${BASE_LINK}/repositories.toml"
    curl -L --output ${ALFRED_DIR}/cron.toml "${BASE_LINK}/cron.toml"
    curl -L --output ${ALFRED_DIR}/routing.toml "${BASE_LINK}/routing.toml"
}

set_service() {
    echo """[Unit]
Description=Alfred Service, which launch daemon and other modules if needed
Wants=network-online.target
After=network.target network-online.target

[Service]
Type=simple
User=${USER}
WorkingDirectory=${ALFRED_DIR}
ExecStart=${ALFRED_DIR}/runner --keep-alive

[Install]
WantedBy=multi-user.target""" > /tmp/alfred.service
sudo mv /tmp/alfred.service /etc/systemd/system/alfred.service
sudo chown root.root /etc/systemd/system/alfred.service
sudo systemctl daemon-reload
sudo systemctl enable alfred
echo "Restart the system or run \`sudo systemctl start alfred\`"
}

echo "Downloading Alfred core from github (${LATEST_VERSION})"
ALFRED_LINK="https://github.com/PaoloPana/alfred-rs/releases/download/${LATEST_VERSION}/alfred-core_${ARCH}.tar.gz"
curl -L --output /tmp/alfred-core.tar.gz ${ALFRED_LINK}

tar -xzf /tmp/alfred-core.tar.gz -C "${INSTALLATION_DIR}/"
# create default config files
if [ ! -f "${ALFRED_DIR}/config.toml" ]; then
    create_default_config
fi
# add service
read -r -p "Do you want to create Alfred service which runs at startup (needs to be root)? [Y/n] " response
case "$response" in
    [nN][oO]|[nN])
        echo "Skipped."
        ;;
    *)
        set_service
        ;;
esac
