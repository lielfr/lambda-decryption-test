FROM amazonlinux:2023

RUN dnf update && dnf install -y zip tar gzip
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.cargo/bin:$PATH"
RUN mkdir /app
WORKDIR /app

COPY . .
RUN chmod +x docker_build.sh
RUN uv python install 3.12
