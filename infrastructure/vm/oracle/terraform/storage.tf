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

  retention_rules {
    display_name = "retention_rule_7_days"
    duration {
      time_amount = 7
      time_unit   = "DAYS"
    }
  }
}

resource "oci_objectstorage_bucket" "loki" {
  compartment_id = oci_identity_compartment.dev.id
  name           = "loki"
  namespace      = data.oci_objectstorage_namespace.namespace.namespace
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = "Disabled"

  retention_rules {
    display_name = "retention_rule_30_days"
    duration {
      time_amount = 30
      time_unit   = "DAYS"
    }
  }
}

resource "oci_objectstorage_bucket" "tempo" {
  compartment_id = oci_identity_compartment.dev.id
  name           = "tempo"
  namespace      = data.oci_objectstorage_namespace.namespace.namespace
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = "Disabled"

  retention_rules {
    display_name = "retention_rule_30_days"
    duration {
      time_amount = 30
      time_unit   = "DAYS"
    }
  }
}

resource "oci_objectstorage_bucket" "vcn_logs" {
  compartment_id = oci_identity_compartment.dev.id
  name           = "vcn_logs"
  namespace      = data.oci_objectstorage_namespace.namespace.namespace
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = "Disabled"

  retention_rules {
    display_name = "retention_rule_30_days"
    duration {
      time_amount = 30
      time_unit   = "DAYS"
    }
  }
}
