FROM ghcr.io/graalvm/native-image-community:21-ol9 AS graalvm
WORKDIR /home/app
COPY --link layers/libs /home/app/libs
COPY --link layers/app /home/app/
COPY --link layers/resources /home/app/resources
RUN mkdir /home/app/config-dirs
RUN mkdir -p /home/app/config-dirs/generateResourcesConfigFile
RUN mkdir -p /home/app/config-dirs/com.fasterxml.jackson.core/jackson-databind/2.15.2
RUN mkdir -p /home/app/config-dirs/io.netty/netty-common/4.1.80.Final
RUN mkdir -p /home/app/config-dirs/org.apache.httpcomponents/httpclient/4.5.14
RUN mkdir -p /home/app/config-dirs/commons-logging/commons-logging/1.2
COPY --link config-dirs/generateResourcesConfigFile /home/app/config-dirs/generateResourcesConfigFile
COPY --link config-dirs/com.fasterxml.jackson.core/jackson-databind/2.15.2 /home/app/config-dirs/com.fasterxml.jackson.core/jackson-databind/2.15.2
COPY --link config-dirs/io.netty/netty-common/4.1.80.Final /home/app/config-dirs/io.netty/netty-common/4.1.80.Final
COPY --link config-dirs/org.apache.httpcomponents/httpclient/4.5.14 /home/app/config-dirs/org.apache.httpcomponents/httpclient/4.5.14
COPY --link config-dirs/commons-logging/commons-logging/1.2 /home/app/config-dirs/commons-logging/commons-logging/1.2
RUN native-image --exclude-config .*/libs/netty-common-4.1.110.Final.jar ^/META-INF/native-image/.* --exclude-config .*/libs/netty-buffer-4.1.110.Final.jar ^/META-INF/native-image/.* --exclude-config .*/libs/netty-codec-http-4.1.110.Final.jar ^/META-INF/native-image/.* --exclude-config .*/libs/netty-transport-4.1.110.Final.jar ^/META-INF/native-image/.* --exclude-config .*/libs/netty-codec-http2-4.1.110.Final.jar ^/META-INF/native-image/.* --exclude-config .*/libs/netty-handler-4.1.110.Final.jar ^/META-INF/native-image/.* -cp /home/app/libs/*.jar:/home/app/resources:/home/app/application.jar --no-fallback -o application -H:ConfigurationFileDirectories=/home/app/config-dirs/generateResourcesConfigFile,/home/app/config-dirs/com.fasterxml.jackson.core/jackson-databind/2.15.2,/home/app/config-dirs/io.netty/netty-codec-http/4.1.80.Final,/home/app/config-dirs/io.netty/netty-common/4.1.80.Final,/home/app/config-dirs/io.netty/netty-buffer/4.1.80.Final,/home/app/config-dirs/io.netty/netty-transport/4.1.80.Final,/home/app/config-dirs/io.netty/netty-handler/4.1.80.Final,/home/app/config-dirs/io.netty/netty-codec-http2/4.1.80.Final,/home/app/config-dirs/org.apache.httpcomponents/httpclient/4.5.14,/home/app/config-dirs/commons-logging/commons-logging/1.2 com.example.FunctionLambdaRuntime
FROM cgr.dev/chainguard/wolfi-base:latest
EXPOSE 8080
COPY --link --from=graalvm /home/app/application /app/application
ENTRYPOINT ["/bin/sh"]
