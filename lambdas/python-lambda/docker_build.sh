#!/bin/bash

uv sync --python-preference only-managed
export SITE_PACKAGES=$(uv pip show boto3 | grep "Location" | sed -e "s/Location: //")

cd $SITE_PACKAGES
zip -rq /dist/lambda.zip .
cd $OLDPWD
zip -rq /dist/lambda.zip .
