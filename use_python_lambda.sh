#!/bin/zsh
cd lambdas/python-lambda
mkdir dist || true
docker build -t python-lambda-builder .
docker run --rm -v ./dist:/dist python-lambda-builder /app/docker_build.sh
cd ../..
mv lambdas/python-lambda/dist/lambda.zip ./python_lambda.zip
