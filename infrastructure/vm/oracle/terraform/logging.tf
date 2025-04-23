# VCN Logging 

resource "oci_logging_log_group" "main-vcn" {
  compartment_id = oci_identity_compartment.dev.id
  display_name   = "main-vcn"
  description    = "VCN flowlogs Loggroup"
}

resource "oci_logging_log" "main-vcn" {
  display_name = "main-vcn"
  log_group_id = oci_logging_log_group.main-vcn.id
  log_type     = "SERVICE"
  is_enabled         = true
  retention_duration = 30
  
  configuration {
    source {
      category    = "vcn"
      resource    = oci_core_vcn.main.id
      service     = "flowlogs"
      source_type = "OCISERVICE"
    }
  }
}

resource "oci_sch_service_connector" "vcn_flowlogs_to_s3" {
  compartment_id = oci_identity_compartment.dev.id
  display_name   = "vcn-flowlogs-to-s3"
  description    = "Forward VCN Flow Logs to S3 bucket"

  source {
    kind = "logging"
    log_sources {
      log_group_id   = oci_logging_log_group.main-vcn.id
      log_id         = oci_logging_log.main-vcn.id
      compartment_id = oci_identity_compartment.dev.id
    }
  }

  target {
    kind   = "objectStorage"
    bucket =  oci_objectstorage_bucket.vcn_logs.name
  }

  # tasks {
  #   kind = "logRule"
  #   condition = <<EOT
  #     data.destinationAdress = '141.253.98.132'
  #   EOT
  # }

  state = "ACTIVE"
}

# resource "oci_identity_policy" "allow_sch_to_write_object_storage" {
#   name           = "AllowSchToWriteToObjectStorage"
#   compartment_id = oci_identity_compartment.dev.id
#   description    = "Allow SCH to write to Object Storage in dev compartment"
#   statements = [
#     "allow any-user to manage objects in compartment id ocid1.compartment.oc1..aaaaaaaa2367n6viggezcqy6db3k3dmojhjxuft262ezobuvdgkxsiudcmca where all {request.principal.type='serviceconnector', target.bucket.name='loki', request.principal.compartment.id='ocid1.compartment.oc1..aaaaaaaa2367n6viggezcqy6db3k3dmojhjxuft262ezobuvdgkxsiudcmca'}"
#   ]
# }

# LB logging

# resource "oci_logging_log_group" "main_lb" {
#   compartment_id = oci_identity_compartment.dev.id
#   display_name   = "main_lb"
#   description = "Load Balancer L4"
# }

# resource "oci_logging_log" "lb_log" {
#   display_name = "log_lb"
#   log_group_id = oci_logging_log_group.main_lb.id
#   log_type     = "SERVICE"

#   configuration {
#     source {
#       category    = "access"
#       resource    =  oci_network_load_balancer_network_load_balancer.example_nlb.id
#       service     = "loadbalancer"
#       source_type = "OCISERVICE"
#     }
#   }

#   is_enabled         = true
#   retention_duration = 30
# }


# resource "oci_core_capture_filter" "test_capture_filter" {
#   compartment_id = oci_identity_compartment.dev.id
#   filter_type    = "FLOWLOG"
#   display_name   = "test_capture_filter"


#   flow_log_capture_filter_rules {
#     rule_action      = "INCLUDE"
#     source_cidr      = "10.0.0.0/16"
#     destination_cidr = "10.0.0.0/16"
#     flow_log_type    = "ALL"
#     is_enabled       = true
#     sampling_rate    = 100000
#   }
# }

