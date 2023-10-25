output "artifact_id" {
  description = "The id of the artifact"
  value       = google_artifact_registry_repository.repo.id
}