locals {
  dev_subnet_cidr_block = "10.0.1.0/24"
}

resource "oci_core_security_list" "public_sl" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "public_sl"

  ingress_security_rules {
    protocol = "1" # ICMP
    source   = "0.0.0.0/0"
    icmp_options {
      type = 8
      code = 0
    }
  }

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
    #source   = local.dev_subnet_cidr_block

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
    source   =  "0.0.0.0/0"

    tcp_options {
      min = 6443
      max = 6443
    }
  }

  # etcd server/client/metrics
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 2379
      max = 2381
    }
  }

  # cilium health
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 4240
      max = 4240
    }
  }

  # Hubble metrics
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 4244
      max = 4244
    }
  }

  # VXLAN 
  ingress_security_rules {
    protocol = "17" # UDP
    source   = local.dev_subnet_cidr_block

    udp_options {
      min = 8472
      max = 8472
    }
  }

  # Node metrics 
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 9100
      max = 9100
    }
  }

  # Metrics
  # 9962 : Cilium
  # 9964 : Cilium Envoy
  # 9965 : Hubble
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 9962
      max = 9965
    }
  }

  # kube-proxy and kubelet metrics
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 10249
      max = 10250
    }
  }

  # controller-manager metrics
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 10257
      max = 10257
    }
  }

  # scheduler metrics
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block

    tcp_options {
      min = 10259
      max = 10259
    }
  }

  # Node Ports
  ingress_security_rules {
    protocol = "6" # TCP
    source   = local.dev_subnet_cidr_block
    #source   = "0.0.0.0/0"

    tcp_options {
        min = 30000
        max = 32767
    }
  }

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }
}
