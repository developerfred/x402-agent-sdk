# Go Gin Test Dockerfile
FROM golang:1.21-alpine AS builder

WORKDIR /app

# Build and test
FROM golang:1.21-alpine

WORKDIR /app

# Copy all source files first
COPY adapters/go/gin/ /app/

# Download dependencies
RUN go mod download github.com/gin-gonic/gin

# Run tests
CMD ["go", "test", "-v", "-cover", "./..."]
