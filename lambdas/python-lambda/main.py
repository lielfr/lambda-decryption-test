import boto3
from aws_lambda_powertools.utilities.typing import LambdaContext
from aws_lambda_powertools import Logger
import os
from cryptography.hazmat.primitives.asymmetric import padding
from cryptography.hazmat.primitives import serialization, hashes

PRIVATE_KEY_PATH = os.environ['PRIVATE_KEY_PATH']
RESULT_BUCKET_PATH_ENV = os.environ['RESULT_BUCKET_PATH']

logger = Logger('python-decryptor', 'DEBUG')
s3 = boto3.client('s3')

secrets_manager_client = boto3.client(service_name='secretsmanager')


def lambda_handler(event: dict, context: LambdaContext):
    logger.debug('initialized Python Lambda handler')

    logger.debug('getting key from Secrets Manager')
    private_key_response = secrets_manager_client.get_secret_value(
        SecretId=PRIVATE_KEY_PATH)
    private_key = private_key_response['SecretString']
    private_key = serialization.load_pem_private_key(
        private_key.encode('utf-8'), password=None)
    logger.debug('got private key successfully')

    for record in event['Records']:
        if 's3' not in record:
            continue
        try:
            bucket = record['s3']['bucket']['name']
            key = record['s3']['object']['key']
            logger.info(f'got bucket: {bucket}, key: {key}, processing')
            item = s3.get_object(Bucket=bucket, Key=key)['Body'].read()
            decrypted_item = private_key.decrypt(
                item,
                padding.OAEP(
                    mgf=padding.MGF1(algorithm=hashes.SHA1()),
                    algorithm=hashes.SHA1(),
                    label=None
                )
            )
            logger.info(
                f'decrypted {bucket}/{key} successfully, writing to target bucket')
            s3.put_object(
                Body=decrypted_item,
                Bucket=RESULT_BUCKET_PATH_ENV,
                Key=key
            )
            logger.info('deleting source object')
            s3.delete_object(Bucket=bucket, Key=key)
            logger.info('finished')
        except Exception as e:
            logger.error(f'Error trying to process record: {e}')
