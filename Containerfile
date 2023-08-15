FROM rust:1.71 as builder
LABEL org.opencontainers.image.source="https://github.com/ARGA-Genomes/arga-backend"
LABEL org.opencontainers.image.description="A container image running the backend server"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

WORKDIR /usr/src/arga-backend
RUN apt-get update && apt-get install -y protobuf-compiler libpq-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo install --path server --locked
RUN cargo install --path workers --locked
RUN cargo install --path tasks --locked

FROM debian:bullseye-slim
LABEL org.opencontainers.image.source="https://github.com/ARGA-Genomes/arga-backend"
LABEL org.opencontainers.image.description="A container image running the backend server"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

ENV SOLR_URL=http://localhost:8983/api
ENV FRONTEND_URL=http://localhost:3000
ENV BIND_ADDRESS=0.0.0.0:5000
EXPOSE 5000
CMD ["arga-backend"]

RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/arga-backend /usr/local/bin/arga-backend
COPY --from=builder /usr/local/cargo/bin/arga-workers /usr/local/bin/arga-workers
COPY --from=builder /usr/local/cargo/bin/arga-tasks /usr/local/bin/arga-tasks
