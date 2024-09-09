package com.example;

import com.amazonaws.services.lambda.runtime.Context;
import com.amazonaws.services.lambda.runtime.LambdaLogger;
import com.amazonaws.services.lambda.runtime.RequestHandler;
import com.amazonaws.services.lambda.runtime.events.S3Event;
import com.amazonaws.services.lambda.runtime.events.models.s3.S3EventNotification;
import com.amazonaws.services.lambda.runtime.logging.LogLevel;
import software.amazon.awssdk.core.ResponseBytes;
import software.amazon.awssdk.core.sync.RequestBody;
import software.amazon.awssdk.services.s3.S3Client;
import software.amazon.awssdk.services.s3.model.DeleteObjectRequest;
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

public class Handler implements RequestHandler<S3Event, Void> {
    final String PRIVATE_KEY_PATH_ENV = "PRIVATE_KEY_PATH";
    final String RESULT_BUCKET_PATH_ENV = "RESULT_BUCKET_PATH";
    @Override
    public Void handleRequest(S3Event s3Event, Context context) {
        LambdaLogger logger = context.getLogger();
        logger.log("S3 event triggered lambda", LogLevel.INFO);
        try {
            SecretsManagerClient secretsManagerClient = SecretsManagerClient.builder().build();
            logger.log("trying to get private key with secrets manager", LogLevel.INFO);
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
            logger.log("got private key successfully", LogLevel.INFO);
            S3Client s3Client = S3Client.builder().build();

            for (S3EventNotification.S3EventNotificationRecord record : s3Event.getRecords()) {
                try {
                    S3EventNotification.S3Entity s3Entity = record.getS3();
                    String bucketName = s3Entity.getBucket().getName();
                    String key = s3Entity.getObject().getKey();
                    logger.log(String.format("processing record. bucket = %s, key=%s\n", bucketName, key), LogLevel.INFO);
                    GetObjectRequest getObjectRequest = GetObjectRequest.builder().bucket(bucketName).key(key).build();
                    ResponseBytes<GetObjectResponse> objectBytes = s3Client.getObjectAsBytes(getObjectRequest);
                    byte[] object = objectBytes.asByteArray();
                    Cipher decryptCipher = Cipher.getInstance("RSA/ECB/OAEPPadding");
                    decryptCipher.init(Cipher.DECRYPT_MODE, privateKey);
                    byte[] decryptedObject = decryptCipher.doFinal(object);
                    logger.log("decryption done, now putting it into target bucket", LogLevel.INFO);
                    PutObjectRequest putObjectRequest = PutObjectRequest.builder().bucket(System.getenv(RESULT_BUCKET_PATH_ENV)).key(key).build();
                    s3Client.putObject(putObjectRequest, RequestBody.fromBytes(decryptedObject));
                    logger.log("successfully put into the target bucket, same key. Now deleting the source object", LogLevel.INFO);
                    s3Client.deleteObject(DeleteObjectRequest.builder().bucket(bucketName).key(key).build());
                    logger.log("successfully deleted the source object", LogLevel.INFO);

                } catch (Exception e) {
                    logger.log("Error while processing record: " + e, LogLevel.INFO);
                }
            }

            s3Client.close();
        } catch (Exception e) {
            logger.log("Error while processing event: " + e, LogLevel.INFO);
        }
        return null;
    }
}
