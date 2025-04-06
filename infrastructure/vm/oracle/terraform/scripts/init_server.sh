#!/bin/bash

/usr/sbin/netfilter-persistent stop
/usr/sbin/netfilter-persistent flush

systemctl stop netfilter-persistent.service
systemctl disable netfilter-persistent.service

apt update
apt upgrade -y
apt install -y nmap