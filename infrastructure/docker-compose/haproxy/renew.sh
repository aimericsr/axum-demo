#!/bin/bash

# We using standalone mode to renew the certificate as no proxy is running on the host on port 80 
# chmod +x renew.sh
# (crontab -l 2>/dev/null; echo "0 0 * * * /root/renew.sh >> /var/log/certbot-renew.log 2>&1") | crontab -

DOMAIN="aimericsorin.me"
EMAIL="aimeric.sorin@gmail.com"
LE_PATH="/etc/letsencrypt/live/$DOMAIN"
HAPROXY_CERT="/etc/haproxy/live/$DOMAIN/$DOMAIN.pem"
DOCKER_HAPROXY_CONTAINER="haproxy" 

if [ ! -f "$LE_PATH/fullchain.pem" ]; then
    echo "No existing certificate found. Requesting a new one..."

    certbot certonly --standalone --agree-tos --non-interactive -d "$DOMAIN" -m "$EMAIL" 

    if [ $? -ne 0 ]; then
        echo "Failed to obtain a new certificate.\n"
        exit 1
    fi
else
    certbot renew --quiet
    if [ $? -ne 0 ]; then
        echo "Certbot renewal failed."
        exit 1
    fi
fi

echo "Certificate is valid. Move file in haproxy compatible format to $HAPROXY_CERT..."
mkdir -p "$(dirname "$HAPROXY_CERT")"
cat "$LE_PATH/cert.pem" "$LE_PATH/privkey.pem" > "$HAPROXY_CERT"
