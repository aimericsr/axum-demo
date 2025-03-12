# Create a VM that is eligible for the free tier
provider "google" {
  credentials = file(var.credentials_file)

  project = var.project_id
  region  = var.region
  zone    = var.zone
}

resource "google_compute_network" "vpc-network-default" {
  name                    = "custom-network"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "subnet-us-central" {
  name          = "subnet-us-central"
  ip_cidr_range = "10.0.0.0/16"
  region        = "us-central1"
  network       = google_compute_network.vpc-network-default.id
}

resource "google_compute_firewall" "allow-ssh" {
  name          = "allow-ssh"
  network       = google_compute_network.vpc-network-default.name
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "tcp"
    ports    = ["22"]
  }
}

resource "google_compute_firewall" "allow-http" {
  name          = "allow-http"
  network       = google_compute_network.vpc-network-default.name
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "tcp"
    ports    = ["80"]
  }
  target_tags = ["http-server"]
}

resource "google_compute_firewall" "allow-https" {
  name          = "allow-https"
  network       = google_compute_network.vpc-network-default.name
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "tcp"
    ports    = ["443"]
  }
  target_tags = ["https-server"]
}

resource "google_compute_firewall" "allow-icmp" {
  name          = "allow-icmp"
  network       = google_compute_network.vpc-network-default.name
  source_ranges = ["0.0.0.0/0"]
  allow {
    protocol = "icmp"
  }
}

resource "google_compute_firewall" "allow-internal" {
  name          = "allow-internal"
  network       = google_compute_network.vpc-network-default.name
  source_ranges = ["10.0.0.0/16"]
  allow {
    protocol = "tcp"
    ports    = ["0-65535"]
  }
  allow {
    protocol = "udp"
    ports    = ["0-65535"]
  }
  allow {
    protocol = "icmp"
  }
}

data "google_compute_default_service_account" "default" {}
resource "google_compute_instance" "main-vm" {
  name         = "main-vm"
  machine_type = "e2-micro"
  zone         = "us-central1-c"

  tags = ["http-server", "https-server"]

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
      size  = 30
    }
  }
  network_interface {
    subnetwork = google_compute_subnetwork.subnet-us-central.name
    access_config {}
  }

  service_account {
    email = data.google_compute_default_service_account.default.email
    scopes = [
      "https://www.googleapis.com/auth/devstorage.read_only",
      "https://www.googleapis.com/auth/logging.write",
      "https://www.googleapis.com/auth/monitoring.write",
      "https://www.googleapis.com/auth/servicecontrol",
      "https://www.googleapis.com/auth/service.management.readonly",
      "https://www.googleapis.com/auth/trace.append"
    ]
  }
}

# Preparing to unpack .../google-cloud-cli_513.0.0-0_amd64.deb ...