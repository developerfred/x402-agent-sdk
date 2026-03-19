# Java Spring Boot Test Dockerfile
FROM eclipse-temurin:17-jdk AS builder

WORKDIR /app

# Install Maven
RUN apt-get update && apt-get install -y maven

# Copy source
COPY adapters/java/springboot/pom.xml /app/pom.xml
COPY adapters/java/springboot/src /app/src

# Run tests
CMD ["mvn", "test", "-Dtest=X402ModelsTest"]
