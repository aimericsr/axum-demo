# data "oci_identity_availability_domains" "ads" {
#   compartment_id = var.tenancy_ocid
# }

# locals {
#   availability_domain_name = data.oci_identity_availability_domains.ads.availability_domains[0].name
#   image_id = data.oci_core_images.test_images.images.0.id
# }

# data "oci_core_images" "test_images" {
#     compartment_id      = oci_identity_compartment.dev.id
#     display_name = "Canonical-Ubuntu-24.04-aarch64-2025.03.28-0"
#     # operating_system = "Canonical Ubuntu"
#     # operating_system_version = "24.04"
# }

# resource "oci_core_instance" "vm1" {
#   display_name        = "vm1"
#   availability_domain = local.availability_domain_name
#   compartment_id      = oci_identity_compartment.dev.id
#   shape               = "VM.Standard.A1.Flex"
#   #shape = "VM.Standard3.Flex"

#   create_vnic_details {
#     subnet_id        = oci_core_subnet.dev.id
#     assign_public_ip = true
#   }

#   source_details {
#     source_type             = "image"
#     source_id               = local.image_id
#     boot_volume_size_in_gbs = 50
#   }

#   shape_config {
#     memory_in_gbs = 6
#     ocpus         = 1
#   }

#   metadata = {
#     ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
#   }

#   preserve_boot_volume = false
# }

# resource "oci_core_instance" "vm2" {
#   display_name        = "vm2"
#   availability_domain = local.availability_domain_name
#   compartment_id      = oci_identity_compartment.dev.id
#   shape               = "VM.Standard.A1.Flex"
#   #shape = "VM.Standard3.Flex"

#   create_vnic_details {
#     subnet_id        = oci_core_subnet.dev.id
#     assign_public_ip = true
#   }

#   source_details {
#     source_type             = "image"
#     source_id               = local.image_id
#     boot_volume_size_in_gbs = 50
#   }

#   shape_config {
#     memory_in_gbs = 6
#     ocpus         = 1
#   }

#   metadata = {
#     ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
#   }

#   preserve_boot_volume = false
# }



resource "oci_core_instance_pool" "k3s_servers" {
  display_name              = "k3s-servers"
  compartment_id            = oci_identity_compartment.dev.id
  instance_configuration_id = oci_core_instance_configuration.k3s_server_template.id

  placement_configurations {
    availability_domain = local.availability_domain_name
    primary_subnet_id   = oci_core_subnet.dev.id
  }

  size = 3
}

data "oci_core_instance_pool_instances" "k3s_servers_instances" {
  depends_on = [
    oci_core_instance_pool.k3s_servers,
  ]
  compartment_id   = oci_identity_compartment.dev.id
  instance_pool_id = oci_core_instance_pool.k3s_servers.id
}

data "oci_core_instance" "k3s_servers_instances_ips" {
  count       = oci_core_instance_pool.k3s_servers.size
  instance_id = data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id
}

output "k3s_servers_private_ips" {
  value = [
    for instance in data.oci_core_instance.k3s_servers_instances_ips :
    instance.private_ip
  ]
}

output "k3s_servers_public_ips" {
  value = [
    for instance in data.oci_core_instance.k3s_servers_instances_ips :
    instance.public_ip
  ]
}