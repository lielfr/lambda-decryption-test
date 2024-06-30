package com.example;
import io.micronaut.function.aws.MicronautRequestHandler;
import com.amazonaws.services.lambda.runtime.events.models.s3.S3EventNotification;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import software.amazon.awssdk.core.ResponseBytes;
import software.amazon.awssdk.core.sync.RequestBody;
import software.amazon.awssdk.services.s3.S3Client;
import software.amazon.awssdk.services.s3.model.GetObjectRequest;
import software.amazon.awssdk.services.s3.model.GetObjectResponse;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.secretsmanager.SecretsManagerClient;
import software.amazon.awssdk.services.secretsmanager.model.GetSecretValueRequest;

import javax.crypto.Cipher;
import java.security.KeyFactory;
import java.security.PrivateKey;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Base64;

public class FunctionRequestHandler extends MicronautRequestHandler<S3EventNotification, Void> {
    final String PRIVATE_KEY_PATH_ENV = "PRIVATE_KEY_PATH";
    final String RESULT_BUCKET_PATH_ENV = "RESULT_BUCKET_PATH";

    @Override
    public Void execute(S3EventNotification input) {
        Logger logger = LoggerFactory.getLogger(FunctionRequestHandler.class);
        logger.info("S3 event triggered lambda");
        try {
            SecretsManagerClient secretsManagerClient = SecretsManagerClient.builder().build();
            logger.info("trying to get private key with secrets manager");
            String privateKeyStr = secretsManagerClient
                    .getSecretValue(GetSecretValueRequest
                            .builder()
                            .secretId(System.getenv(PRIVATE_KEY_PATH_ENV))
                            .build())
                    .secretString().replaceAll("\\n", "").replace("-----BEGIN PRIVATE KEY-----", "")
                    .replace("-----END PRIVATE KEY-----", "");
            PKCS8EncodedKeySpec privateKeySpec = new PKCS8EncodedKeySpec(Base64.getDecoder().decode(privateKeyStr));
            KeyFactory keyFactory = KeyFactory.getInstance("RSA");
            PrivateKey privateKey = keyFactory.generatePrivate(privateKeySpec);
            logger.info("got private key successfully");
            S3Client s3Client = S3Client.builder().build();

            for (S3EventNotification.S3EventNotificationRecord record : input.getRecords()) {
                try {
                    S3EventNotification.S3Entity s3Entity = record.getS3();
                    String bucketName = s3Entity.getBucket().getName();
                    String key = s3Entity.getObject().getKey();
                    logger.info(String.format("processing record. bucket = %s, key=%s\n", bucketName, key));
                    GetObjectRequest getObjectRequest = GetObjectRequest.builder().bucket(bucketName).key(key).build();
                    ResponseBytes<GetObjectResponse> objectBytes = s3Client.getObjectAsBytes(getObjectRequest);
                    byte[] object = objectBytes.asByteArray();
                    Cipher decryptCipher = Cipher.getInstance("RSA/ECB/OAEPPadding");
                    decryptCipher.init(Cipher.DECRYPT_MODE, privateKey);
                    byte[] decryptedObject = decryptCipher.doFinal(object);
                    logger.info("decryption done, now putting it into target bucket");
                    PutObjectRequest putObjectRequest = PutObjectRequest.builder().bucket(System.getenv(RESULT_BUCKET_PATH_ENV)).key(key).build();
                    s3Client.putObject(putObjectRequest, RequestBody.fromBytes(decryptedObject));
                    logger.info("successfully put into the target bucket, same key.");

                } catch (Exception e) {
                    logger.error("Error while processing record: " + e);
                }
            }
        } catch (Exception e) {
            logger.error("Error while processing event: " + e);
        }
        return null;
    }
}
