resource "oci_identity_compartment" "dev" {
  compartment_id = var.root_compartment_id
  name           = "dev"
  description    = "VCN for dev projects"
  enable_delete  = true
}




