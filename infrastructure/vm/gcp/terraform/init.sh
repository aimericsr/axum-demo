#!/bin/bash
set -e 

# to run this script : sudo bash init.sh

# echo "Updating package list..."
# apt-mark hold google-cloud-cli 
# apt update && apt upgrade -y

echo "Installing necessary packages for Docker ..."
apt install ca-certificates curl -y

echo "Set up Docker's apt repository ..."
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
apt update

echo "Install Docker ..."
apt install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y

echo "Post install Docker ..."
groupadd docker
usermod -aG docker $(whoami)
systemctl enable docker.service
systemctl enable containerd.service

# echo "Remove existing docker installation..."
# for pkg in docker.io docker-doc docker-compose podman-docker containerd runc; do apt purge $pkg; done
# rm -rf /var/lib/docker
# rm -rf /var/lib/containerd
# rm /etc/apt/sources.list.d/docker.list
# rm /etc/apt/keyrings/docker.asc

apt install openssl git -y