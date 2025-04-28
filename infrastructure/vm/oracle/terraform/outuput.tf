output "k3s_servers_private_ips" {
  value = [
    for instance in local.k3s_control_planes :
    instance.private_ip
  ]
}

output "k3s_servers_public_ips" {
  value = [
    for instance in local.k3s_control_planes :
    instance.public_ip
  ]
}

output "k3s_ocids" {
  value = [
    for instance in local.k3s_control_planes :
    instance.id
  ]
}

output "k3s_lb_public_ip" {
  value = [
    for ip in oci_network_load_balancer_network_load_balancer.k3s_lb.ip_addresses :
    ip.ip_address if ip.is_public
  ][0]
}


output "etcd_backdup_s3_compatible_endpoint" {
  value = "${data.oci_objectstorage_namespace.namespace.namespace}.compat.objectstorage.${var.region}.oraclecloud.com"
}

output "etcd_backdup_s3_compatible_bucket_name" {
  value = oci_objectstorage_bucket.etcd_backup.name
}

output "compartment_id" {
  value = oci_identity_compartment.dev.id
}


output "vcn_id" {
  value = oci_core_vcn.main.id
}

# output "ads" {
#   value = data.oci_identity_availability_domains.ads.availability_domains[0].name
# }

