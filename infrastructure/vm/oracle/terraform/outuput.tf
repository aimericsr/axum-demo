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

output "k3s_lb_public_ip" {
  value = [
    for ip in oci_network_load_balancer_network_load_balancer.example_nlb.ip_addresses :
    ip.ip_address if ip.is_public
  ][0]
}

output "s3_compatible_endpoint" {
  value = "${data.oci_objectstorage_namespace.namespace.namespace}.compat.objectstorage.${var.region}.oraclecloud.com"
}

output "s3_compatible_bucket_name" {
  value = oci_objectstorage_bucket.test_bucket.name
}