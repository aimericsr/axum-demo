# resource "oci_network_load_balancer_network_load_balancer" "example_nlb" {
#   compartment_id = oci_identity_compartment.dev.id
#   subnet_id      = oci_core_subnet.dev.id

#   display_name   = "kubernetes-cluster"
#   is_private     = false
#   is_preserve_source_destination = true
# }

# resource "oci_network_load_balancer_listener" "listener_http" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   name                     = "http_listener"
#   protocol                 = "TCP"
#   port                     = 80
#   default_backend_set_name = oci_network_load_balancer_backend_set.http_backend_set.name
# }

# resource "oci_network_load_balancer_listener" "listener_https" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   name                     = "https_listener"
#   protocol                 = "TCP"
#   port                     = 443
#   default_backend_set_name = oci_network_load_balancer_backend_set.https_backend_set.name
# }

# resource "oci_network_load_balancer_listener" "listener_k8s" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   name                     = "k8s_listener"
#   protocol                 = "TCP"
#   port                     = 6443
#   default_backend_set_name = oci_network_load_balancer_backend_set.k8s_backend_set.name
# }

# resource "oci_network_load_balancer_backend_set" "http_backend_set" {
#   name                       = "http_backend_set"
#   network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   policy                     = "FIVE_TUPLE"

#   health_checker {
#     port     = 80
#     protocol = "TCP"
#   }
# }

# resource "oci_network_load_balancer_backend_set" "https_backend_set" {
#   name                       = "https_backend_set"
#   network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   policy                     = "FIVE_TUPLE"

#   health_checker {
#     port     = 443
#     protocol = "TCP"
#   }
# }

# resource "oci_network_load_balancer_backend_set" "k8s_backend_set" {
#   name                       = "k8s_backend_set"
#   network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   policy                     = "FIVE_TUPLE"

#   health_checker {
#     port     = 6443
#     protocol = "TCP"
#   }
# }

# # VM1
# resource "oci_network_load_balancer_backend" "backend_1_http" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
#   ip_address               = "10.0.1.30"
#   port                     = 80
# }

# resource "oci_network_load_balancer_backend" "backend_1_https" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
#   ip_address               = "10.0.1.30"
#   port                     = 443
# }

# resource "oci_network_load_balancer_backend" "backend_1_k8s" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.k8s_backend_set.name
#   ip_address               = "10.0.1.30"
#   port                     = 6443
# }

# #VM2
# resource "oci_network_load_balancer_backend" "backend_2_http" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
#   ip_address               = "10.0.1.14"
#   port                     = 80
# }

# resource "oci_network_load_balancer_backend" "backend_2_https" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
#   ip_address               = "10.0.1.14"
#   port                     = 443
# }

# resource "oci_network_load_balancer_backend" "backend_2_k8s" {
#   network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
#   backend_set_name         = oci_network_load_balancer_backend_set.k8s_backend_set.name
#   ip_address               = "10.0.1.14"
#   port                     = 6443
# }
