#!/usr/bin/env sh

docker exec -it kafka-1 /opt/bitnami/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --create --if-not-exists --topic messages --replication-factor 1 --partitions 1
