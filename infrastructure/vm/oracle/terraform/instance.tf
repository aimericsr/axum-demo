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