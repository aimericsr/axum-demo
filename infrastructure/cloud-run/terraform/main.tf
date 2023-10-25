# gcloud services enable run.googleapis.com --quiet
# gcloud services enable artifactregistry.googleapis.com --quiet
# gcloud services enable cloudtrace.googleapis.com --quiet
# gcloud services enable monitoring.googleapis.com --quiet

# gcloud auth configure-docker \
#     us-central1-docker.pkg.dev

#docker build -t us-central1-docker.pkg.dev/sandbox-381015/green-harvest/web-api  --platform=linux/amd64 . 

terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "5.3.0"
    }
  }
}

provider "google" {
  credentials = file(var.credentials_file)

  project     = var.project_id
  region      = var.gcp_region
  zone    =     var.gcp_zone
}

resource "google_service_account" "github_sa" {
  account_id   = var.sa_name
  display_name = "Cloud Run OpenTelemetry demo service account"
  description  = "A service account just to be used for Cloud Run observability demo. https://github.com/GoogleCloudPlatform/opentelemetry-cloud-run"
}

resource "google_project_iam_binding" "sa_iam_binding1" {
  project     = var.project_id
  role    = "roles/iam.serviceAccountUser"
  members = ["serviceAccount:${google_service_account.github_sa.email}"]
}

resource "google_project_iam_binding" "sa_iam_binding2" {
  project     = var.project_id
  role    = "roles/cloudtrace.agent"
  members = ["serviceAccount:${google_service_account.github_sa.email}"]
}

resource "google_project_iam_binding" "sa_iam_binding3" {
  project     = var.project_id
  role    = "roles/logging.logWriter"
  members = ["serviceAccount:${google_service_account.github_sa.email}"]
}

resource "google_project_iam_binding" "sa_iam_binding4" {
  project     = var.project_id
  role    = "roles/artifactregistry.createOnPushWriter"
  members = ["serviceAccount:${google_service_account.github_sa.email}"]
}

resource "google_project_iam_binding" "sa_iam_binding5" {
  project     = var.project_id
  role    = "roles/run.admin"
  members = ["serviceAccount:${google_service_account.github_sa.email}"]
}

resource "google_artifact_registry_repository" "repo" {
  repository_id   = var.repository_id
  location        = var.gcp_region
  format          = "DOCKER"
}
