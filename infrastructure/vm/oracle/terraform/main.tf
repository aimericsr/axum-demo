resource "oci_identity_compartment" "dev" {
  compartment_id = var.root_compartment_id
  name           = "dev"
  description    = "VCN for dev projects"
  enable_delete  = true
}


# resource "oci_core_volume" "test_volume" {
# 	compartment_id = oci_identity_compartment.dev.compartment_id

# 	availability_domain = local.availability_domain_name

# 	display_name = "main-disk"
# 	size_in_gbs = 50
# }

# resource "oci_core_volume_attachment" "tf_volume_attachment" {
#     attachment_type = "iscsi"
#     instance_id     = oci_core_instance.vm1.id
#     volume_id       = oci_core_volume.test_volume.id
# }






