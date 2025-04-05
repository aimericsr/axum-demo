# Create free tier vms

# Create RSA keys for auth against oracle APIs: 
# - mkdir $HOME/.oci
# - openssl genrsa -out $HOME/.oci/id_rsa.pem 2048
# - chmod 600 $HOME/.oci/id_rsa.pem
# - openssl rsa -pubout -in $HOME/.oci/id_rsa.pem -out $HOME/.oci/id_rsa_public.pem
# - cat $HOME/.oci/id_rsa_public.pem
provider "oci" {
  tenancy_ocid     = var.tenancy_ocid
  user_ocid        = var.user_ocid
  fingerprint      = var.fingerprint
  private_key_path = var.private_key_path
  region           = var.region
}

resource "oci_identity_compartment" "dev" {
  compartment_id = var.root_compartment_id
  name           = "dev"
  description = "VCN for dev projects"
  enable_delete  = true
}

# Network
resource "oci_core_vcn" "main" {
  compartment_id = oci_identity_compartment.dev.id
  display_name   = "main-vcn"
  cidr_block     = "10.0.0.0/16"
  dns_label      = "vcnmain"
}

resource "oci_core_internet_gateway" "main_internet_gateway" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  enabled        = true
  display_name   = "main_internet_gateway"
}

resource "oci_core_route_table" "public_rt" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "Public-Route-Table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.main_internet_gateway.id
  }
}

resource "oci_core_security_list" "public_sl" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "public_sl"

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"

    tcp_options {
      min = 22
      max = 22
    }
  }

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"

    tcp_options {
      min = 80
      max = 80
    }
  }

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"

    tcp_options {
      min = 443
      max = 443
    }
  }

  ingress_security_rules {
    protocol    = "1" # ICMP
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"

    icmp_options {
      type = 8 # Echo Request (ping)
      code = 0 # Any code
    }
  }

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"

    tcp_options {
      min = 6443
      max = 6443
    }
  }

  egress_security_rules {
    protocol         = "all"
    destination      = "0.0.0.0/0"
    destination_type = "CIDR_BLOCK"
  }
}

resource "oci_core_security_list" "private_sl" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "private_sl"

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "10.0.1.0/24"
    source_type = "CIDR_BLOCK"

    tcp_options {
      min = 2379
      max = 2380
    }
  }
}

resource "oci_core_subnet" "dev" {
  vcn_id            = oci_core_vcn.main.id
  cidr_block        = "10.0.1.0/24"
  display_name      = "dev"
  dns_label         = "subnetdev"
  route_table_id    = oci_core_route_table.public_rt.id
  security_list_ids = [oci_core_security_list.public_sl.id, oci_core_security_list.private_sl.id]
  compartment_id    = oci_identity_compartment.dev.id
}

data "oci_identity_availability_domains" "ads" {
  compartment_id = var.tenancy_ocid
}

data "oci_core_images" "ubuntu-24_04" {
  compartment_id           = oci_identity_compartment.dev.id
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "24.04"
}

locals {
  availability_domain_name = data.oci_identity_availability_domains.ads.availability_domains[0].name
  os_image                 = data.oci_core_images.ubuntu-24_04.images.3.id
}

# resource "oci_core_volume" "test_volume" {
# 	compartment_id = oci_identity_compartment.dev.compartment_id

# 	availability_domain = local.availability_domain_name

# 	display_name = "main-disk"
# 	size_in_gbs = 50
# }

# resource "oci_core_volume_attachment" "tf_volume_attachment" {
#     attachment_type = "iscsi"
#     instance_id     = oci_core_instance.vm1.id
#     volume_id       = oci_core_volume.test_volume.id
# }

resource "oci_core_instance" "vm1" {
  display_name        = "vm1"
  availability_domain = local.availability_domain_name
  compartment_id      = oci_identity_compartment.dev.id
  #shape               = "VM.Standard.A1.Flex"
  shape = "VM.Standard3.Flex"

  create_vnic_details {
    subnet_id        = oci_core_subnet.dev.id
    assign_public_ip = true
  }

  source_details {
    source_type             = "image"
    source_id               = local.os_image
    boot_volume_size_in_gbs = 50
  }

  shape_config {
    memory_in_gbs = 6
    ocpus         = 1
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
  }

  preserve_boot_volume = false
}

resource "oci_core_instance" "vm2" {
  display_name        = "vm2"
  availability_domain = local.availability_domain_name
  compartment_id      = oci_identity_compartment.dev.id
  #shape               = "VM.Standard.A1.Flex"
  shape = "VM.Standard3.Flex"

  create_vnic_details {
    subnet_id        = oci_core_subnet.dev.id
    assign_public_ip = true
  }

  source_details {
    source_type             = "image"
    source_id               = local.os_image
    boot_volume_size_in_gbs = 50
  }

  shape_config {
    memory_in_gbs = 6
    ocpus         = 1
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
  }

  preserve_boot_volume = false
}

data "oci_objectstorage_namespace" "namespace" {
  compartment_id = oci_identity_compartment.dev.id
}

resource "oci_objectstorage_bucket" "test_bucket" {
  compartment_id = oci_identity_compartment.dev.id
  name           = "etcd"
  namespace      = data.oci_objectstorage_namespace.namespace.namespace
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = "Disabled"
}

# LB
resource "oci_network_load_balancer_network_load_balancer" "example_nlb" {
  compartment_id = oci_identity_compartment.dev.id
  subnet_id      = oci_core_subnet.dev.id

  display_name   = "kubernetes-cluster"
  is_private     = false
  is_preserve_source_destination = true
}

# Backends
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

resource "oci_network_load_balancer_backend_set" "k8s_backend_set" {
  name                       = "k8s_backend_set"
  network_load_balancer_id   = oci_network_load_balancer_network_load_balancer.example_nlb.id
  policy                     = "FIVE_TUPLE"

  health_checker {
    port     = 6443
    protocol = "TCP"
  }
}

# VM1
resource "oci_network_load_balancer_backend" "backend_1_http" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
  ip_address               = "10.0.1.30"
  port                     = 80
}

resource "oci_network_load_balancer_backend" "backend_1_https" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
  ip_address               = "10.0.1.30"
  port                     = 443
}

resource "oci_network_load_balancer_backend" "backend_1_k8s" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.k8s_backend_set.name
  ip_address               = "10.0.1.30"
  port                     = 6443
}

#VM2
resource "oci_network_load_balancer_backend" "backend_2_http" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.http_backend_set.name
  ip_address               = "10.0.1.14"
  port                     = 80
}

resource "oci_network_load_balancer_backend" "backend_2_https" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.https_backend_set.name
  ip_address               = "10.0.1.14"
  port                     = 443
}

resource "oci_network_load_balancer_backend" "backend_2_k8s" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  backend_set_name         = oci_network_load_balancer_backend_set.k8s_backend_set.name
  ip_address               = "10.0.1.14"
  port                     = 6443
}

# Listeners
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

resource "oci_network_load_balancer_listener" "listener_k8s" {
  network_load_balancer_id = oci_network_load_balancer_network_load_balancer.example_nlb.id
  name                     = "k8s_listener"
  protocol                 = "TCP"
  port                     = 6443
  default_backend_set_name = oci_network_load_balancer_backend_set.k8s_backend_set.name
}

