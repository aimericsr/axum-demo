terraform {
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "6.32.0"
    }
    cloudinit = {
      source  = "hashicorp/cloudinit"
      version = "2.3.5"
    }
  }
}