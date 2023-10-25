variable "credentials_file" {
  type        = string
  description = "Path to the service account credentials JSON file for GCP authentication."
}

variable "project_id" {
  type        = string
  description = "The ID of the GCP project where the resources will be created."
}

variable "gcp_region" {
  type        = string
  description = "The GCP region"
  default     = "us-central1"
}

variable "gcp_zone" {
  type        = string
  description = "The GCP zone"
  default     = "us-central1-a"
}

variable "sa_name" {
  type        = string
  description = "The name of the service account"
  default     = "github-ci"
}

variable "repository_id" {
    type        = string
  description = "The name of the repository"
  default     = "green-harvest"
}