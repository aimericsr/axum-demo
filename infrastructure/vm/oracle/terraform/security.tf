resource "oci_core_security_list" "public_sl" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "public_sl"

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 22
      max = 22
    }
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 80
      max = 80
    }
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 443
      max = 443
    }
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 6443
      max = 6443
    }
  }

    ingress_security_rules {
    protocol = "6" # TCP
    source   = "10.0.1.0/24"

    tcp_options {
      min = 2379
      max = 2380
    }
  }

  ingress_security_rules {
    protocol = "17" # UDP
    source   = "10.0.1.0/24"

    udp_options {
      min = 8472
      max = 8472
    }
  }

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }
}
