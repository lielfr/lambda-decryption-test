go-lambda:
	cd lambdas/go-lambda; \
	GOOS=linux GOARCH=arm64 CGO_ENABLED=0 go build -tags lambda.norpc -o bootstrap main.go; \
	zip ../../go_lambda.zip bootstrap; \
	rm bootstrap; \
	cd ../..

java-lambda:
	cd lambdas/java-lambda; \
	./gradlew clean buildNativeLambda; \
	cd ../..; \
	cp lambdas/java-lambda/build/libs/java-lambda-0.1-lambda.zip ./java_lambda.zip

jvm-lambda:
	cd lambdas/jvm-lambda; \
	./gradlew clean packageJar; \
	cd ../..; \
	cp lambdas/jvm-lambda/build/distributions/jvm-lambda-1.0-SNAPSHOT.zip ./jvm_lambda.zip

python-lambda:
	cd lambdas/python-lambda; \
	mkdir dist || true; \
	uv export | uv pip install --target dist --python-platform aarch64-unknown-linux-gnu -r -; \
	cd dist; \
	cp ../main.py .; \
	zip -rq ../lambda.zip .; \
	cd ../../..;\
	rm -rf lambdas/python-lambda/dist; \
	mv lambdas/python-lambda/lambda.zip ./python_lambda.zip

rust-lambda:
	cd lambdas/rust-lambda; \
	cargo lambda build --release --arm64; \
	cd ../..; \
	cp lambdas/rust-lambda/target/lambda/rust-lambda/bootstrap .; \
	zip -r rust_lambda.zip bootstrap; \
	rm bootstrap

nodejs-lambda:
	cd lambdas/nodejs-lambda; \
	zip -r ../../nodejs_lambda.zip index.mjs; \
	cd ../..

all:
	go-lambda java-lambda jvm-lambda python-lambda rust-lambda
