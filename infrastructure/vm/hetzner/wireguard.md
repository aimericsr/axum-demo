# Server
apt install wireguard
vi /etc/sysctl.conf -> uncomment net.ipv4.ip_forward
sysctl -p
cd /etc/wireguard
umask 077
wg genkey > server_priv
wg genkey < server_priv > server_pub

vi wg0.conf
[Interface]
Adress = 134.3.3.2/24
ListenPort = 60000
PrivateKey =

[Peer]
PublicKey = 
AllowedIPs = 134.3.3.2/24
[PersistentKeepalive]

wg-quick up wg0
ip a show dev wg0

# Client
umask 077
wg genkey > client_priv
wg genkey < client_priv > client_pub

vi wg0.conf
[Interface]
Adress = 134.3.3.2/24
ListenPort = 60000
PrivateKey =

[Peer]
PublicKey = 
AllowedIPs = 134.3.3.2/24
Endpoint = 134.3.3.2:60000

wg-quick up wg0
ip a show dev wg0