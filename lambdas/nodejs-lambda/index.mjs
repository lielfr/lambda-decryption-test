import {
  DeleteObjectCommand,
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import {
  GetSecretValueCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";
import * as crypto from "node:crypto";

const PRIVATE_KEY_PATH_ENV = "PRIVATE_KEY_PATH";
const RESULT_BUCKET_PATH_ENV = "RESULT_BUCKET_PATH";

const smClient = new SecretsManagerClient();
const s3Client = new S3Client();

export const handler = async (event, context) => {
  console.log("started Node.JS Lambda");

  try {
    console.log("getting private key from secrets manager");
    const response = await smClient.send(
      new GetSecretValueCommand({
        SecretId: process.env[PRIVATE_KEY_PATH_ENV],
      }),
    );
    const privateKeyStr = response.SecretString;
    if (!privateKeyStr) {
      throw new Error("could not get private key");
    }

    const privateKey = crypto.createPrivateKey(privateKeyStr);

    for (const record of event.Records) {
      const bucket = record.s3.bucket.name;
      const key = decodeURIComponent(record.s3.object.key.replace(/\+/g, " "));

      console.log(`processing object: bucket=${bucket}, key=${key}`);
      try {
        const objectRequest = await s3Client.send(
          new GetObjectCommand({
            Bucket: bucket,
            Key: key,
          }),
        );
        if (!objectRequest.Body) {
          throw new Error("object body is empty");
        }
        const source = await objectRequest.Body.transformToByteArray();
        console.log("got object, proceeding to decrypt");
        const result = crypto.privateDecrypt(
          {
            key: privateKey,
          },
          Buffer.from(source),
        );
        console.log("decrypted successfully, now uploading to target bucket");
        await s3Client.send(
          new PutObjectCommand({
            Bucket: process.env[RESULT_BUCKET_PATH_ENV],
            Key: key,
            Body: result,
          }),
        );

        console.log(
          "uploaded decrypted file successfully, now deleting original object from source bucket",
        );
        await s3Client.send(
          new DeleteObjectCommand({
            Bucket: bucket,
            Key: key,
          }),
        );

        console.log("finished processing file");
      } catch (err) {
        console.error(`could not process object, skipping! Error: ${err}`);
        continue;
      }
    }
  } catch (err) {
    console.error(`got error: ${err}`);
    throw new Error(err);
  }

  console.log("finished running NodeJS Lambda");
};
