# Lambdas Benchmark

This is the accompanying repository for my article about benchmarking various Lambda functions.

## Running

You'll need to first build all of the lambdas. For this, you'll need:

1. Stable Rust
2. cargo-lambda
3. GraalVM
4. OpenJDK v21
5. Go
6. Docker (or alternatives like Podman)
7. Python 3.12
8. uv (Python package manager)

You can build everything in one go by running `make`, or you can pick your desired target(s). See `Makefile` for targets list.

In order to deploy to the cloud, you'll need Terraform. `cd` into the infra directory, and run `terraform apply`.
Due to a bug in the AWS provider, you might need to do this more than once.

Once everything is deployed, you can use the util to generate test data and upload it to the required buckets. The results should be available in CloudWatch.

After everything is done, make sure you destroy the provisioned resources by running `terraform destroy` inside the infra directory.
