# resource "oci_bastion_bastion" "k3s_nodes" {
#     compartment_id = oci_identity_compartment.dev.id
#     target_subnet_id = oci_core_subnet.dev.id
#     bastion_type = "STANDARD"
#     name = "k3s-nodes"
#     client_cidr_block_allow_list = [ "0.0.0.0/0"]
# }

# resource "oci_bastion_session" "session_forward_ssh" {
#   bastion_id = oci_bastion_bastion.k3s_nodes.id
#   display_name           = "session-forward-ssh"
#   key_type               = "PUB"
#   session_ttl_in_seconds = 10800

#   key_details {
#     public_key_content = file("~/.ssh/id_ed25519.pub")
#   }

#   target_resource_details {
#     session_type       = "PORT_FORWARDING"
#     target_resource_id = oci_core_instance.k3s_control_plane_1.id
#     target_resource_port = 22
#   }
# }

# output "session" {
#     value = oci_bastion_session.session_forward_ssh.ssh_metadata["command"]
# }