resource "oci_core_vcn" "main" {
  compartment_id = oci_identity_compartment.dev.id
  display_name   = "main-vcn"
  cidr_block     = "10.0.0.0/16"
  dns_label      = "vcnmain"
}

resource "oci_core_subnet" "dev" {
  compartment_id             = oci_identity_compartment.dev.id
  vcn_id                     = oci_core_vcn.main.id
  cidr_block                 = "10.0.1.0/24"
  display_name               = "dev"
  dns_label                  = "subnetdev"
  route_table_id             = oci_core_route_table.public_rt.id
  security_list_ids          = [oci_core_security_list.public_sl.id]
  prohibit_public_ip_on_vnic = false
}

resource "oci_core_subnet" "dev2" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  cidr_block     = "10.0.2.0/24"
  display_name   = "dev2"
  dns_label      = "subnetdev2"
  route_table_id = oci_core_route_table.private_rt.id
  #security_list_ids = [oci_core_security_list.public_sl.id]
}

resource "oci_core_internet_gateway" "main_internet_gateway" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  enabled        = true
  display_name   = "main_internet_gateway"
}

resource "oci_core_nat_gateway" "main_nat_gateway" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "main_nat_gateway"
}

resource "oci_core_service_gateway" "main_service_gateway" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "main_service_gateway"

  services {
    service_id = data.oci_core_services.all_services.services[0].id
  }
}

data "oci_core_services" "all_services" {
  filter {
    name   = "name"
    values = ["All CDG Services In Oracle Services Network"]
    regex  = true
  }
}

resource "oci_core_route_table" "public_rt" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "public-rt"

  route_rules {
    description       = "Allow incoming internet packets"
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.main_internet_gateway.id
  }
}

resource "oci_core_route_table" "private_rt" {
  compartment_id = oci_identity_compartment.dev.id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "private-rt"

  route_rules {
    description       = "Route to Oracle Services via Service Gateway"
    destination       = data.oci_core_services.all_services.services[0].cidr_block
    destination_type  = "SERVICE_CIDR_BLOCK"
    network_entity_id = oci_core_service_gateway.main_service_gateway.id
  }

  route_rules {
    description       = "Route to Internet via NAT"
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_nat_gateway.main_nat_gateway.id
  }
}

#  6.8.0-1022-oracle