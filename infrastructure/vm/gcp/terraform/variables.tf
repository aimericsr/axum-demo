variable "credentials_file" {
  type        = string
  description = "Path to the service account credentials JSON file for GCP authentication."
  default     = "~/credentials/gcp/key.json"
}

variable "project_id" {
  type        = string
  description = "The ID of the GCP project where the resources will be created."
  default     = "sandbox-381015"
}

variable "region" {
  type        = string
  description = "The GCP region"
  default     = "us-central1"
}

variable "zone" {
  type        = string
  description = "The GCP zone"
  default     = "us-central1-c"
}