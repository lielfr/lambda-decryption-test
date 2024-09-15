package main

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha1"
	"crypto/x509"
	"io"
	"log"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go-v2/config"

	"encoding/pem"
	"os"

	"github.com/aws/aws-sdk-go-v2/service/s3"

	"github.com/aws/aws-sdk-go-v2/service/secretsmanager"
)

const SECRET_ENV = "SECRET"
const TARGET_BUCKET_ENV = "TARGET_BUCKET"

func HandleRequest(ctx context.Context, event *events.S3Event) error {
	log.Println("starting Go lambda")
	sdkConfig, err := config.LoadDefaultConfig(ctx)
	if err != nil {
		log.Printf("failed to load default config: %s\n", err)
		return err
	}
	smClient := secretsmanager.NewFromConfig(sdkConfig)
	privKeySecretPath := os.Getenv(SECRET_ENV)
	privKeyResult, err := smClient.GetSecretValue(ctx, &secretsmanager.GetSecretValueInput{SecretId: &privKeySecretPath})
	if err != nil {
		log.Printf("failed to get private key secret: %s\n", err)
		return err
	}
	privKeyPem, _ := pem.Decode([]byte(*privKeyResult.SecretString))
	privKey, err := x509.ParsePKCS1PrivateKey(privKeyPem.Bytes)
	if err != nil {
		log.Printf("failed to parse private key: %s\n", err)
		return err
	}
	log.Println("got private key successfully")
	targetBucketName := os.Getenv(TARGET_BUCKET_ENV)

	s3Client := s3.NewFromConfig(sdkConfig)
	for _, record := range event.Records {
		bucket := record.S3.Bucket.Name
		key := record.S3.Object.URLDecodedKey
		log.Printf("processing: bucket = %s, key = %s\n", bucket, key)
		obj, err := s3Client.GetObject(ctx, &s3.GetObjectInput{Bucket: &bucket, Key: &key})
		if err != nil {
			log.Printf("could not process object: %s\n", err)
			continue
		}
		objBody := new(bytes.Buffer)
		_, err = io.Copy(objBody, obj.Body)
		if err != nil {
			log.Printf("could not read object: %s\n", err)
			continue
		}
		hash := sha1.New()

		decryptedObj, err := rsa.DecryptOAEP(hash, rand.Reader, privKey, objBody.Bytes(), nil)
		if err != nil {
			log.Printf("could not decrypt object: %s\n", err)
			continue
		}
		decryptedObjBody := bytes.NewBuffer(decryptedObj)

		log.Println("decrypted successfully, now uploading to target bucket")
		_, err = s3Client.PutObject(ctx, &s3.PutObjectInput{
			Bucket: &targetBucketName,
			Key:    &key,
			Body:   decryptedObjBody,
		})
		if err != nil {
			log.Printf("could not upload result to S3: %s\n", err)
			continue
		}

		log.Println("uploaded successfully, now deleting original object")
		_, err = s3Client.DeleteObject(ctx, &s3.DeleteObjectInput{
			Bucket: &bucket,
			Key:    &key,
		})

		if err != nil {
			log.Printf("could not delete original object from S3: %s\n", err)
			continue
		}

		log.Printf("finished handling object (bucket=%s, key=%s)\n", bucket, key)

	}
	log.Println("finished running Go lambda handler")
	return nil

}

func main() {
	lambda.Start(HandleRequest)
}
