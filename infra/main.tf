module "rust_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../rust_lambda.zip"
  unique_identifier  = "lielfr_rust_lambda_test"
  source_bucket_name = "lambda-source-bucket-rust"
  target_bucket_name = "lambda-target-bucket-rust"
  memory_size        = 3008
}

module "java_lambda" {
  source             = "./modules/test_config"
  lambda_zip         = "../java_lambda.zip"
  unique_identifier  = "lielfr-java-lambda-test"
  source_bucket_name = "lambda-source-bucket-java"
  target_bucket_name = "lambda-target-bucket-java"
  memory_size        = 3008
}
