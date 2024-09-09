#!/bin/zsh
cd lambdas/python-lambda
mkdir dist || true
uv export | uv pip install --target dist --python-platform aarch64-unknown-linux-gnu -r -
cd dist
cp ../main.py .
zip -rq ../lambda.zip .
cd ../../..
rm -rf lambdas/python-lambda/dist
mv lambdas/python-lambda/lambda.zip ./python_lambda.zip
