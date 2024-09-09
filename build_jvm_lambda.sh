#!/bin/zsh
cd lambdas/jvm-lambda
./gradlew clean packageJar
cd ../..
cp lambdas/jvm-lambda/build/distributions/jvm-lambda-1.0-SNAPSHOT.zip ./jvm_lambda.zip
