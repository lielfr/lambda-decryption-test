variable "target_bucket_name" {
  type = string
}

variable "source_bucket_name" {
  type = string
}

variable "unique_identifier" {
  type = string
}

variable "lambda_zip" {
  type = string
}

variable "memory_size" {
  type    = number
  default = 128
}

variable "runtime" {
  type    = string
  default = "provided.al2023"
}

variable "handler" {
  type    = string
  default = "bootstrap.App"
}
