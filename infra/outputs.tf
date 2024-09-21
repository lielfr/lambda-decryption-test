output "rust_target_bucket" {
  value = module.rust_lambda.source_bucket
}

output "java_target_bucket" {
  value = module.java_lambda.source_bucket
}

output "python_target_bucket" {
  value = module.python_lambda.source_bucket
}

output "jvm_target_bucket" {
  value = module.jvm_lambda.source_bucket
}

output "go_target_bucket" {
  value = module.go_lambda.source_bucket
}

output "nodejs_target_bucket" {
  value = module.nodejs_lambda.source_bucket
}
