## kafka-scylla-micro
A repository that aims to showcase a toy microservice that uses Kafka & ScyllaDB (with basic CDC usage).

This repository is mostly a WIP but will be used to showcase some of my gRPC/Kafka learnings. 

## How to use
- Use `docker compose up -d` to spin up the infrastructure.
- Use `./buyer/kafka.sh` to initialise the Kafka topic.
- Use `KAFKA_URL=localhost:9092 cargo run -p messenger` to run the only available Rust program at the moment.
