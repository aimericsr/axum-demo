variable "tenancy_ocid" {
  type        = string
  description = "The OCID of your tenancy."
  default     = "ocid1.tenancy.oc1..aaaaaaaa2nimi5bnhsi5zoyxtbtglkcumccjfmkc5ajsoftlrd4m7krhtsda"
}

variable "user_ocid" {
  type        = string
  description = "The OCID of your user."
  default     = "ocid1.user.oc1..aaaaaaaai2anfuh4aadc63vceu27mrub7u74n2oyisz7hajetncxu4eiheeq"
}

variable "fingerprint" {
  type        = string
  description = "The fingerprint for your API key."
  default     = "e7:69:19:16:1c:2e:ac:80:2e:c4:97:eb:61:4a:0a:63"
}

variable "private_key_path" {
  type        = string
  description = "Path to your private key file."
  default     = "~/.oci/id_rsa.pem"
}

variable "region" {
  type        = string
  description = "The region where your OCI resources will be created."
  default     = "eu-paris-1"
}

variable "root_compartment_id" {
  type        = string
  description = "The root compartment id"
  default     = "ocid1.tenancy.oc1..aaaaaaaa2nimi5bnhsi5zoyxtbtglkcumccjfmkc5ajsoftlrd4m7krhtsda"
}

variable "image_id" {
  type        = string
  description = "The image id for VMs"
  default     = "ocid1.tenancy.oc1..aaaaaaaa2nimi5bnhsi5zoyxtbtglkcumccjfmkc5ajsoftlrd4m7krhtsda"
}