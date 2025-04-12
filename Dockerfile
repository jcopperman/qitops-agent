FROM rust:1.77-slim as builder

WORKDIR /usr/src/qitops-agent
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/qitops-agent/target/release/qitops /usr/local/bin/qitops

# Create config directory
RUN mkdir -p /root/.config/qitops

# Set environment variables
ENV GITHUB_TOKEN=""
ENV OPENAI_API_KEY=""

ENTRYPOINT ["qitops"]
CMD ["--help"]
