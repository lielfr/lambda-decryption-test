#!/bin/sh
cd lambdas/go-lambda
GOOS=linux GOARCH=arm64 CGO_ENABLED=0 go build -tags lambda.norpc -o bootstrap main.go
zip ../../go_lambda.zip bootstrap
rm bootstrap
cd ../..
