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
COPY ~/.cargo/bin/arga-backend /usr/local/bin/arga-backend
