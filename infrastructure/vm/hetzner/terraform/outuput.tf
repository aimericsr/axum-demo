output "vm_public_ipv4" {
  description = "The public IP of the VM"
  value       = hcloud_server.main_vm.ipv4_address 
}