FROM rust:1.67 as builder
WORKDIR /usr/src/arga-backend
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/arga-backend /usr/local/bin/arga-backend

ENV SOLR_URL=http://localhost:8983/api
ENV FRONTEND_URL=http://localhost:3000
ENV BIND_ADDRESS=0.0.0.0:5000
EXPOSE 5000
CMD ["arga-backend"]

