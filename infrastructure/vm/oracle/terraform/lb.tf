# k3s Admin LB
resource "oci_network_load_balancer_network_load_balancer" "k3s_admin_lb" {
  depends_on = [
    local.k3s_control_planes,
  ]

  compartment_id                 = oci_identity_compartment.dev.id
  subnet_id                      = oci_core_subnet.dev.id
  display_name                   = "k3s_admin_lb"
  is_private                     = false
  is_preserve_source_destination = false
}

resource "oci_network_load_balancer_listener" "listener_k3s" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_admin_lb.id
  name                     = "listener-k3s"
  protocol                 = "TCP"
  port                     = 6443
  default_backend_set_name = oci_network_load_balancer_backend_set.k3s_backend_set.name
  is_ppv2enabled           = false
}

resource "oci_network_load_balancer_backend_set" "k3s_backend_set" {
  name                     = "k3s-backend-set"
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_admin_lb.id
  policy                   = "FIVE_TUPLE"
  is_preserve_source = false

  health_checker {
    port     = 6443
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend" "k3s_backend" {
  depends_on = [
    local.k3s_control_planes
  ]

  count                    = length(local.k3s_control_planes)
  backend_set_name         = oci_network_load_balancer_backend_set.k3s_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_admin_lb.id
  name                     = format("%s:%s", local.k3s_control_planes[count.index].id, 6443)
  port                     = 6443
  target_id                = local.k3s_control_planes[count.index].id
}

# Apps LB
resource "oci_network_load_balancer_network_load_balancer" "k3s_apps_lb" {
  depends_on = [
    local.k3s_control_planes,
  ]

  compartment_id                 = oci_identity_compartment.dev.id
  subnet_id                      = oci_core_subnet.dev.id
  display_name                   = "k3s-apps-lb"
  is_private                     = false
  is_preserve_source_destination = false

}

resource "oci_network_load_balancer_listener" "listener_http" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  name                     = "listener-http"
  protocol                 = "TCP"
  port                     = 80
  default_backend_set_name = oci_network_load_balancer_backend_set.http_backend_set.name
  is_ppv2enabled           = false
}

resource "oci_network_load_balancer_listener" "listener_https" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  name                     = "listener-https"
  protocol                 = "TCP"
  port                     = 443
  default_backend_set_name = oci_network_load_balancer_backend_set.https_backend_set.name
  is_ppv2enabled           = false
}

resource "oci_network_load_balancer_backend_set" "http_backend_set" {
  name                     = "http-backend-set"
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  policy                   = "FIVE_TUPLE"
  is_preserve_source = false

  health_checker {
    port     = 30282
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend_set" "https_backend_set" {
  name                     = "https-backend-set"
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  policy                   = "FIVE_TUPLE"
  is_preserve_source = false

  health_checker {
    port     = 31660
    protocol = "TCP"
  }
}

resource "oci_network_load_balancer_backend" "http_backend" {
  depends_on = [
    local.k3s_control_planes,
  ]

  count                    = length(local.k3s_control_planes)
  backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  name                     = format("%s:%s", local.k3s_control_planes[count.index].id, 80)
  port                     = 30282
  target_id                = local.k3s_control_planes[count.index].id
}

resource "oci_network_load_balancer_backend" "https_backend" {
  depends_on = [
    local.k3s_control_planes,
  ]

  count                    = length(local.k3s_control_planes)
  backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.k3s_apps_lb.id
  name                     = format("%s:%s", local.k3s_control_planes[count.index].id, 443)
  port                     = 31660
  target_id                = local.k3s_control_planes[count.index].id
}

# resource "oci_core_public_ip" "k3s_admin_lb_ip" {
#   compartment_id = oci_identity_compartment.dev.id
#   lifetime       = "RESERVED"
#   display_name   = "k3s-admin-lb-ip"
#   #private_ip_id  = oci_core_private_ip.test_private_ip.id
#   #private_ip_id = oci_core_private_ip.test_private_ip.id
# }

# resource "oci_core_private_ip" "test_private_ip" {
#   #display_name = "test_private_ip"
#   #lifetime     = "RESERVED"
#   #subnet_id    = oci_core_subnet.dev.id

#   subnet_id      = oci_core_subnet.dev.id
#   display_name   = "available_reserve_private_ip"
#   route_table_id = oci_core_route_table.main_rt.id
#   lifetime       = "RESERVED"
# }