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