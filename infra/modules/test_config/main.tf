terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "eu-central-1"
  default_tags {
    tags = {
      Project = "lambdas_test"
    }
  }
}

resource "aws_s3_bucket" "source" {
  bucket = var.source_bucket_name
}

resource "aws_s3_bucket" "target" {
  bucket = var.target_bucket_name
}

resource "aws_secretsmanager_secret" "private-key" {
  name = "decryption-lambdas-test-private-key-3-${var.unique_identifier}"
}

resource "aws_secretsmanager_secret_version" "private-key" {
  secret_id     = aws_secretsmanager_secret.private-key.id
  secret_string = file("../private.pem")
}

resource "aws_iam_policy" "lambda_policy" {
  name = "lambda-policy-${var.unique_identifier}"
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Effect = "Allow"
        Resource = [
          aws_secretsmanager_secret.private-key.arn
        ]
      },
      {
        Action = [
          "s3:GetObject"
        ]
        Effect = "Allow"
        Resource = [
          "${aws_s3_bucket.source.arn}/*"
        ]
      },
      {
        Action = [
          "s3:PutObject"
        ]
        Effect = "Allow"
        Resource = [
          "${aws_s3_bucket.target.arn}/*"
        ]
      },
    ]
  })
}

data "aws_iam_policy_document" "assume_role" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

data "aws_iam_policy" "lambda_execution_role" {
  arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role" "iam_for_lambda" {
  name               = "iam_for_lambda-${var.unique_identifier}"
  assume_role_policy = data.aws_iam_policy_document.assume_role.json
}

resource "aws_lambda_function" "s3-lambda" {
  filename         = var.lambda_zip
  runtime          = "provided.al2023"
  function_name    = "decryption-lambda-test2-${var.unique_identifier}"
  role             = aws_iam_role.iam_for_lambda.arn
  handler          = "bootstrap.App"
  architectures    = ["arm64"]
  source_code_hash = filesha512(var.lambda_zip)
  memory_size      = var.memory_size
  environment {
    variables = {
      PRIVATE_KEY_PATH   = aws_secretsmanager_secret.private-key.name
      RESULT_BUCKET_PATH = aws_s3_bucket.target.bucket
    }
  }
}

resource "aws_iam_role_policy_attachment" "role-policy" {
  policy_arn = aws_iam_policy.lambda_policy.arn
  role       = aws_iam_role.iam_for_lambda.name
}

resource "aws_iam_policy_attachment" "lambda-role-policy-basic-execution-role" {
  policy_arn = data.aws_iam_policy.lambda_execution_role.arn
  roles      = [aws_iam_role.iam_for_lambda.name]
  name       = "role-policy-attachment-lambda-decryption-basic-execution-${var.unique_identifier}"
}

resource "aws_lambda_permission" "invoke-permission-s3" {
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.s3-lambda.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.source.arn
}

resource "aws_s3_bucket_notification" "bucket-notification" {
  bucket = aws_s3_bucket.source.id

  lambda_function {
    lambda_function_arn = aws_lambda_function.s3-lambda.arn
    events              = ["s3:ObjectCreated:*"]
  }
}
