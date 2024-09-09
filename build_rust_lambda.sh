#!/bin/zsh
cd lambdas/rust-lambda
cargo lambda build --release --arm64
cd ../..
cp lambdas/rust-lambda/target/lambda/rust-lambda/bootstrap .
zip -r rust_lambda.zip bootstrap
rm bootstrap
