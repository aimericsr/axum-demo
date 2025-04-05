data "oci_objectstorage_namespace" "namespace" {
  compartment_id = oci_identity_compartment.dev.id
}

# resource "oci_objectstorage_bucket" "test_bucket" {
#   compartment_id = oci_identity_compartment.dev.id
#   name           = "etcd"
#   namespace      = data.oci_objectstorage_namespace.namespace.namespace
#   access_type    = "NoPublicAccess"
#   storage_tier   = "Standard"
#   versioning     = "Disabled"
# }