[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

# Web API to demonstrate axum capabilities

[Axum](https://github.com/tokio-rs/axum) is a web framework that is supported by the tokio team. It integrate natively with the tower and tower-http ecosystem, so we can use middleware, services, and utilities with axum. Moreover, we can also use this with other framework that work with tower like hyper for http and tonic for grpc.

The internal logic is intentionally keep simple. This allow to focus on the overwall logic of the Axum framework, it's observability and the deployment part. As this app is a demo, the goal is to be compliant against the most web API standards.

The infrastructure folder contain code to be able to run the applications in differentes environnements : [Docker Compose](https://docs.docker.com/compose/), [Cloud Run](https://cloud.google.com/run) and [Kubernetes](https://kubernetes.io).

## Kubernetes Architecture

The observability architecture is based on the official [exemple](https://opentelemetry.io/docs/demo/architecture/) of the opentelemetry website exept that we are not using the prometheus pushgateway but the pull mechanisme.

The architecture is using kubernetes to simulate a production environmment using [minikube](https://minikube.sigs.k8s.io/docs/start/).

```mermaid
graph LR;
 client([Client])-. Ingress-managed <br> load balancer .->ingress[Ingress];
 subgraph cluster
 ingress-->|routing rule|service_web[Web API Service];

 subgraph app[Application]
 service_web-->pod_web_1[Pod];
 end

 pod_web_1-->service_database[Database Service];
 pod_web_1-->|send traces|otel_collector[Otel Collector];

 subgraph otel[OpenTelemetry Collector]
 otel_collector;
 end
 otel_collector-->service_jaeger[Service];

 subgraph jaeger[Jaeger Backend]
 service_jaeger-->pod_jaeger_1[Pod];
 end

 subgraph database[Database]
 service_database-->pod_database_1[Pod];
 service_database_exporter[Database Exporter Service]-->|fetch metrics, convert it<br/>and expose to /metrics|service_database[Database Service];
 end

 subgraph prometheus_operator[Prometheus Operator]
 service_prometheus-->pod_prometheus_1[Pod];
 end

 service_prometheus[Prometheus Service]-->|scrape /metrics|service_web;
 service_prometheus[Prometheus Service]-->|scrape /metrics|service_database_exporter[Database Exporter Service];

 grafana[Grafana UI]-->|fetch data|service_prometheus;
 grafana[Grafana UI]-->|fetch data|service_jaeger;

 end
 classDef plain fill:#ddd,stroke:#fff,stroke-width:4px,color:#000;
 classDef k8s fill:#326ce5,stroke:#fff,stroke-width:4px,color:#fff;
 classDef box fill:#fff,stroke:#bbb,stroke-width:2px,color:#326ce5;
 class ingress,service_web,service_database,service_database_exporter,service_prometheus,otel_collector,grafana,service_jaeger,pod_web_1,pod_database_1,pod_jaeger_1,pod_prometheus_1 k8s;
 class client plain;
 class cluster,prometheus_operator,jaeger,database,app,otel box;
```

The prometheus operator is used to monitor kubernetes API, the postgres instance and the web application. The database is generally hosted outside of the kubernetes cluster but for the sake of the demonstration, we will deploy it inside the cluster. To add a new target to prometheus, we juste have to create a CRD of type ServiceMonitor and add the label:
release: prometheus.

The opentelemetry operator give use new CRD for opentelmetry collectors with differents deployments modes(deployment, statefulset, daemonset, sidecar). Here we use the daemonset mode to be sure that one pod is available on all nodes. The collector is able to receive data from different sources, here we send it via grpc on the port 4317 on the collector. It then process data to add kubernetes metadata for all traces it receive to enriche the information of the traces and add more contexte.

All of the traces are then exported to a backend : jaeger. All the traces will be store here and we can view it with the jaeger UI. There are also backends type for cloud integrations like cloud trace for GCP or AWS X-Ray. A full list of the available exporters can be found [here](https://opentelemetry.io/ecosystem/registry/?component=exporter)

## Exemple of a trace view in jaeger UI :

![screenshot of jaeger](.github/assets/trace_exemple.png)

We have the information from the application like the http route, thread.
And the infrastructure information added by the otel collector like the kubernetes namespace, pod name, pod creation.
With that, we can correlate application traces with prometeuses metrics from kubernetes.

## Features

- [RFC 7807](https://datatracker.ietf.org/doc/html/rfc7807) compliant for all error message
- [OpenAPI](https://www.openapis.org) docs
- Traces export to an otel backend in the [otlp](https://opentelemetry.io) format
- Push metrics via otlp protocol to otlp collector, then metrics are send to prometheus via prometheusremotewrite
- Graceful Shutdown for sending last traces before the application stop
- Visualize data with [grafana](https://grafana.com)
- Timout
- Rate Limiting
- Serve static file
- Health check routes
- Rest Routes
- RPC routes
- Auth with cookies and jwt

## To Do

- All the list of TODO can be found [here](https://github.com/users/aimericsr/projects/1)
- fix db related tests
- load env variable only at the beginning ?
- version the api (path or url ?)
- handle db connection retry system
- Infra : Deployments strategies and upgrade helm charts ?

## Prerequesite

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/engine/install/)(also install Docker compose)
- [Minikube](https://minikube.sigs.k8s.io/docs/start/) (also install kubectl)
- [Helm](https://helm.sh/docs/intro/install/)
- [k6](https://k6.io/docs/get-started/installation/)
- [cmctl](https://cert-manager.io/docs/reference/cmctl/#installation)
- [opensl](https://openssl-library.org/source/)

## Create self-signed certificate
```sh
# For all platforms
mkdir -p ~/ssl && cd ~/ssl
openssl req -x509 -out my-app.crt -keyout my-app.key \
  -newkey rsa:2048 -nodes -sha256 \
  -subj '/CN=my-app' -extensions EXT -config <( \
   printf "[dn]\nCN=my-app\n[req]\ndistinguished_name = dn\n[EXT]\nsubjectAltName=DNS:my-app\nkeyUsage=digitalSignature\nextendedKeyUsage=serverAuth")
cat my-app.crt my-app.key > my-app.pem

# Then add the certs to you system store and DNS resolver (system dependant)
# Ex: MacOS
echo "127.0.0.1 my-app" >> /etc/hosts
# Add cert but not trusted
sudo security add-certificates -k /Library/Keychains/System.keychain ~/ssl/my-app.crt   
# Add trusted cert, password is needed
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/ssl/my-app.crt

curl -v -L my-app http://my-app/health/ready
```

## Starting the needed services

```sh
docker compose --profile dev up -d
```

## Tests (Unit, Integration, Doc)

```sh
cargo test -- --nocapture
```

## Docs

```sh
cargo doc --document-private-items --open
```

## Project Conventions

function test name : test*[function_name]*[ok/err]\_[case_tested]

Prefix PR names: 
docs, fix, feat, ci, chore, perf, refactor, test


## Rust

```sh
brew install openssl@1.1
cargo install cargo-edit
cargo install cargo-expand
cargo install --version=0.7.2 sqlx-cli --no-default-features --features postgres
```

## Run load

```sh
docker-compose --profile load-test run k6 run -o experimental-prometheus-rw /scripts/script.js
```

## Minikube

```sh
minikube start --memory 4096 --cpus 4
minikube addons enable dashboard
minikube addons enable metrics-server
minikube addons enable ingress
# minikube tunnel (ingress available at 127.0.0.1)
```

To use Ingress on local with a host add the following line to your /etc/hosts file: <br> 127.0.0.1 host-name-you-want

## Start k8s cluster

```sh
# Create namespace
kubectl apply -f infrastructure/kubernetes/app/namespaces/dev.yaml
kubectl config set-context minikube --namespace=dev

# Helm : add needed repositories
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
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

# Install Cert manager
helm install \
  cert-manager jetstack/cert-manager \
  --namespace cert-manager \
  --create-namespace \
  --version v1.13.1 \
  --set installCRDs=true

# Test the install of cert manager
cmctl check api --wait=2m

# Install the opentelemetry Operator, this automatically generate a self-signed cert and a secret for the webhook
helm install my-opentelemetry-operator open-telemetry/opentelemetry-operator --version "0.39.1" \
    -f infrastructure/kubernetes/helm/opentelemetry-operator/values.yaml

# Create custom CRD for otlp collectors
kubectl apply -f infrastructure/kubernetes/app/rbac/otel-dev.yaml
kubectl apply -R -f infrastructure/kubernetes/app/opentelemetrycollectors

# Create the ressources for our applications
kubectl apply -R -f infrastructure/kubernetes/app/configmaps
kubectl apply -R -f infrastructure/kubernetes/app/secrets
kubectl apply -R -f infrastructure/kubernetes/app/services
kubectl apply -R -f infrastructure/kubernetes/app/statefulsets
kubectl apply -R -f infrastructure/kubernetes/app/deployments
kubectl apply -R -f infrastructure/kubernetes/app/ingresses

# Create Service account
kubectl apply -f infrastructure/kubernetes/app/rbac/github-ci.yaml
TOKEN=$(kubectl create token github-ci)

# Extract info from default minikube profile
SERVER_ADDRESS=$(kubectl config view -o jsonpath='{.clusters[?(@.name == "minikube")].cluster.server}')
CA_CERT=$(kubectl config view -o jsonpath='{.clusters[?(@.name == "minikube")].cluster.certificate-authority}')
# User
kubectl config set-credentials github-ci --token=$TOKEN
# Cluster
kubectl config set-cluster github-ci --server=$SERVER_ADDRESS
kubectl config set-cluster github-ci --certificate-authority=$CA_CERT
# Context
kubectl config set-context github-ci --user=github-ci
kubectl config set-context github-ci --cluster=github-ci
kubectl config set-context github-ci --namespace=dev
kubectl config use-context github-ci
kubectl config get-contexts
```

