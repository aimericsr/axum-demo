provider "hcloud" {
  token = trimspace(file(var.credentials_file))
}

resource "hcloud_ssh_key" "main" {
  name       = "main"
  public_key = file("~/.ssh/id_ed25519.pub")
}

resource "hcloud_server" "main_vm" {
  name        = "main-vm"
  image       = "debian-12"
  server_type = "cx22"
  datacenter = "nbg1-dc3"
  public_net {
    ipv4_enabled = true
    ipv6_enabled = false
  }
  ssh_keys = [ hcloud_ssh_key.main.id ]
}