# Build and test
FROM golang:1.21-alpine

WORKDIR /app

COPY adapters/go/fiber/ /app/

RUN go mod download github.com/gofiber/fiber/v2

CMD ["go", "test", "-v", "-cover", "./..."]
