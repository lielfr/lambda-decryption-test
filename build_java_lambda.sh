#!/bin/zsh
cd lambdas/java-lambda
./gradlew clean buildNativeLambda
cd ../..
cp lambdas/java-lambda/build/libs/java-lambda-0.1-lambda.zip ./java_lambda.zip
