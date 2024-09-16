variable "memory_size" {
  type    = number
  default = 128
}

module "rust_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../rust_lambda.zip"
  unique_identifier  = "lielfr_rust_lambda_test"
  source_bucket_name = "lambda-source-bucket-rust"
  target_bucket_name = "lambda-target-bucket-rust"
  memory_size        = var.memory_size
}

module "java_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../java_lambda.zip"
  unique_identifier  = "lielfr-java-lambda-test"
  source_bucket_name = "lambda-source-bucket-java"
  target_bucket_name = "lambda-target-bucket-java"
  memory_size        = var.memory_size
}

module "python_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../python_lambda.zip"
  unique_identifier  = "lielfr-python-lambda-test"
  source_bucket_name = "lambda-source-bucket-python"
  target_bucket_name = "lambda-target-bucket-python"
  runtime            = "python3.12"
  handler            = "main.lambda_handler"
  memory_size        = var.memory_size
}

module "jvm_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../jvm_lambda.zip"
  unique_identifier  = "lielfr-jvm-lambda-test"
  source_bucket_name = "lambda-source-bucket-jvm"
  target_bucket_name = "lambda-target-bucket-jvm"
  runtime            = "java21"
  handler            = "com.example.Handler"
  memory_size        = var.memory_size
}

module "go_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../go_lambda.zip"
  unique_identifier  = "lielfr-go-lambda-test"
  source_bucket_name = "lambda-source-bucket-go"
  target_bucket_name = "lambda-target-bucket-go"
  handler            = "bootstrap"
  memory_size        = var.memory_size
}
