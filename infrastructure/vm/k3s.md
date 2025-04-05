
# Modify default iptables rules to allow agent to registered against the control plane
sudo vi /etc/iptables/rules.v4
-A INPUT -p tcp -m state --state NEW -m tcp --dport 2379 -j ACCEPT
-A INPUT -p tcp -m state --state NEW -m tcp --dport 2380 -j ACCEPT
-A INPUT -p tcp -m state --state NEW -m tcp --dport 6443 -j ACCEPT

# Issue
https://github.com/tailscale/tailscale/issues/13863



sudo iptables-restore < /etc/iptables/rules.v4

# Install k3s
## Simple
We can install a kube cluster with only one server that run the controle plan and containerd worklowd

curl -sfL https://get.k3s.io | sh -s - server \
    --tls-san=<LB_IP>

curl -sfL https://get.k3s.io | K3S_TOKEN=<SECRET> sh -s - agent \
    --server https://<IP_OR_DNS_SERVER1>:6443 \
    --tls-san=<LB_IP>

## HA cluster Embedded etcd
For the clean install : 

curl -sfL https://get.k3s.io | sh -s - server \
    --cluster-init \
    --tls-san=<LB_IP> 
    
   # \
   # --etcd-s3 \
   # --etcd-s3-bucket=<S3-BUCKET-NAME> \
   # --etcd-s3-access-key=<S3-ACCESS-KEY> \
   # --etcd-s3-secret-key=<S3-SECRET-KEY>

curl -sfL https://get.k3s.io | K3S_TOKEN=<SECRET> sh -s - server \
    --server https://<IP_OR_DNS_SERVER1>:6443 \
    --tls-san=<LB_IP>


curl -sfL https://get.k3s.io | K3S_TOKEN=<df> sh -s - server \
    --server https://10.0.1.30:6443 \
    --tls-san=158.178.200.24

Add this commands if you want to restore etcd database from S3 file : 
  --cluster-reset \
  --cluster-reset-restore-path=<PATH-TO-SNAPSHOT>

## HA cluster External DB

sudo cat /var/lib/rancher/k3s/server/token

sudo cat /etc/rancher/k3s/k3s.yaml

mkdir -p $HOME/.kube && cd $HOME/.kube
vi config
kubectl config use default
kubectl config get-contexts

mkdir $HOME/snapchot
sudo k3s etcd-snapshot save --data-dir $HOME/snapchot

https://web.archive.org/web/20240726111518/https://prog.world/is-storage-speed-suitable-for-etcd-ask-fio/
sudo apt install -y fio
fio --rw=write --ioengine=sync --fdatasync=1 --size=22m --bs=2300 --name etcd
99.00th should be under 10 000 (10 ms)

# Install commen helm charts

helm list --all-namespaces

helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --version "4.12.1" \
  --namespace ingress-nginx \
  --create-namespace

helm upgrade --install cert-manager cert-manager \
  --repo https://charts.jetstack.io \
  --version v1.17.1 \
  --namespace cert-manager \
  --create-namespace 
#--set crds.enabled=true

k3s server --cluster-reset

cmctl check api --wait=2m

helm repo add jetstack https://charts.jetstack.io
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts

# Install Prometheus to scrape kubernetes engine metrics, install also Grafana with build-in dashboard
helm install prometheus prometheus-community/kube-prometheus-stack --version "51.2.0" \
    -f infrastructure/kubernetes/helm/kube-prometheus-stack/values.yaml \
    --namespace=dev

# Create postgres exporter to be able to monitor with prometheus
helm install postgres-exporter prometheus-community/prometheus-postgres-exporter --version "5.1.0" \
    -f infrastructure/kubernetes/helm/prometheus-postgres-exporter/values.yaml \
    --namespace=dev




# Install the opentelemetry Operator, this automatically generate a self-signed cert and a secret for the webhook
helm install my-opentelemetry-operator open-telemetry/opentelemetry-operator --version "0.39.1" \
    -f infrastructure/kubernetes/helm/opentelemetry-operator/values.yaml

