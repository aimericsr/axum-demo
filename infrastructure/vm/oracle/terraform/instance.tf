resource "oci_core_instance" "k3s_control_plane_1" {
  availability_domain = local.availability_domain_name
  compartment_id      = oci_identity_compartment.dev.id
  display_name        = "k3s-control-plane-1"

  shape = "VM.Standard.A1.Flex"
  shape_config {
    memory_in_gbs = "6"
    ocpus         = "2"
  }

  create_vnic_details {
    assign_public_ip = true
    subnet_id        = oci_core_subnet.dev.id
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
    user_data           = data.cloudinit_config.k3s_server.rendered
  }

  source_details {
    source_type             = "image"
    source_id               = local.image_id
    boot_volume_size_in_gbs = 50
  }

  preserve_boot_volume = false
}

resource "oci_core_instance" "k3s_control_plane_2" {
  availability_domain = local.availability_domain_name
  compartment_id      = oci_identity_compartment.dev.id
  display_name        = "k3s-control-plane-2"

  shape = "VM.Standard.A1.Flex"
  shape_config {
    memory_in_gbs = "6"
    ocpus         = "1"
  }

  create_vnic_details {
    assign_public_ip = true
    subnet_id        = oci_core_subnet.dev.id
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
    user_data           = data.cloudinit_config.k3s_server.rendered
  }

  source_details {
    source_type             = "image"
    source_id               = local.image_id
    boot_volume_size_in_gbs = 50
  }

  preserve_boot_volume = false
}

resource "oci_core_instance" "k3s_control_plane_3" {
  availability_domain = local.availability_domain_name
  compartment_id      = oci_identity_compartment.dev.id
  display_name        = "k3s-control-plane-3"

  shape = "VM.Standard.A1.Flex"
  shape_config {
    memory_in_gbs = "6"
    ocpus         = "1"
  }

  create_vnic_details {
    assign_public_ip = true
    subnet_id        = oci_core_subnet.dev.id
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_ed25519.pub")
    user_data           = data.cloudinit_config.k3s_server.rendered
  }

  source_details {
    source_type             = "image"
    source_id               = local.image_id
    boot_volume_size_in_gbs = 50
  }
  
  preserve_boot_volume = false
}

locals {
  k3s_control_planes = [
    oci_core_instance.k3s_control_plane_1,
    oci_core_instance.k3s_control_plane_2,
    oci_core_instance.k3s_control_plane_3,
  ]
}

locals {
  availability_domain_name = data.oci_identity_availability_domains.ads.availability_domains[0].name
  image_id                 = data.oci_core_images.ubuntu.images.0.id
}

data "oci_identity_availability_domains" "ads" {
  compartment_id = var.tenancy_ocid
}

data "oci_core_images" "ubuntu" {
  compartment_id = oci_identity_compartment.dev.id
  display_name   = "Canonical-Ubuntu-24.04-aarch64-2025.03.28-0"
  # operating_system = "Canonical Ubuntu"
  # operating_system_version = "24.04"
}

data "cloudinit_config" "k3s_server" {
  gzip          = true
  base64_encode = true

  part {
    content_type = "text/x-shellscript"
    content      = file("${path.module}/scripts/init_server.sh")
  }
}