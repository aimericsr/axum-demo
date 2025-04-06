# Create a public L4 network load balancer to expose port 80,443 and 6443 and forward traffic into the 3-nodes server cluster.
resource "oci_network_load_balancer_network_load_balancer" "example_nlb" {
  compartment_id = oci_identity_compartment.dev.id
  subnet_id      = oci_core_subnet.dev.id

  display_name   = "kubernetes-cluster"
  is_private     = false
  is_preserve_source_destination = false
}

resource "oci_network_load_balancer_listener" "listener_http" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = "http_listener"
  protocol                 = "TCP"
  port                     = 80
  default_backend_set_name = oci_network_load_balancer_backend_set.http_backend_set.name
}

resource "oci_network_load_balancer_listener" "listener_https" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = "https_listener"
  protocol                 = "TCP"
  port                     = 443
  default_backend_set_name = oci_network_load_balancer_backend_set.https_backend_set.name
}

resource "oci_network_load_balancer_listener" "listener_k3s" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = "k8s_listener"
  protocol                 = "TCP"
  port                     = 6443
  default_backend_set_name = oci_network_load_balancer_backend_set.k3s_backend_set.name
}

resource "oci_network_load_balancer_backend_set" "http_backend_set" {
  name                       = "http_backend_set"
  network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
  policy                     = "FIVE_TUPLE"

  health_checker {
    port     = 80
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend_set" "https_backend_set" {
  name                       = "https_backend_set"
  network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
  policy                     = "FIVE_TUPLE"

  health_checker {
    port     = 443
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend_set" "k3s_backend_set" {
  name                       = "k8s_backend_set"
  network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
  policy                     = "FIVE_TUPLE"

  health_checker {
    port     = 6443
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend" "http_backend" {
  depends_on = [
    oci_core_instance_pool.k3s_servers,
  ]

  count                    = oci_core_instance_pool.k3s_servers.size
  backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = format("%s:%s", data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id, 80)
  port                     = 80
  target_id                = data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id
}

resource "oci_network_load_balancer_backend" "https_backend" {
  depends_on = [
    oci_core_instance_pool.k3s_servers,
  ]

  count                    = oci_core_instance_pool.k3s_servers.size
  backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = format("%s:%s", data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id, 443)
  port                     = 443
  target_id                = data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id
}

resource "oci_network_load_balancer_backend" "k3s_backend" {
  depends_on = [
    oci_core_instance_pool.k3s_servers,
  ]

  count                    = oci_core_instance_pool.k3s_servers.size
  backend_set_name         = oci_network_load_balancer_backend_set.k3s_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = format("%s:%s", data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id, 6443)
  port                     = 6443
  target_id                = data.oci_core_instance_pool_instances.k3s_servers_instances.instances[count.index].id
}
