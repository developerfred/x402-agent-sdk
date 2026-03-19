# Java Quarkus Test Dockerfile
FROM maven:3.9-eclipse-temurin-17 AS builder

WORKDIR /app

COPY adapters/java/quarkus/pom.xml /app/pom.xml
COPY adapters/java/quarkus/src /app/src

RUN mvn dependency:go-offline

CMD ["mvn", "test"]
