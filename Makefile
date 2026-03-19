# x402 Agent SDK - Makefile for running tests
# Usage: make <target>

# Colors
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

.PHONY: help test-all test-python test-go test-java test-rust test-dotnet test-ruby test-php test-core

# Default target
help:
	@echo ""
	@echo "${BLUE}x402 Agent SDK - Test Runner${NC}"
	@echo ""
	@echo "Usage: ${GREEN}make <target>${NC}"
	@echo ""
	@echo "${YELLOW}Available targets:${NC}"
	@echo "  test-all       Run all tests (requires Docker)"
	@echo "  test-core      Run Rust core tests (cargo test)"
	@echo "  test-python    Run Python adapter tests"
	@echo "  test-go        Run Go adapter tests"
	@echo "  test-java      Run Java adapter tests"
	@echo "  test-rust      Run Rust adapter tests"
	@echo "  test-dotnet    Run .NET adapter tests"
	@echo "  test-ruby      Run Ruby adapter tests"
	@echo "  test-php       Run PHP adapter tests"
	@echo "  docker-build   Build all Docker test images"
	@echo "  docker-clean   Clean up Docker containers"
	@echo ""

# Run all tests via Docker
test-all:
	@echo "${GREEN}Running all tests via Docker...${NC}"
	cd adapters && docker-compose up

# Core Rust tests
test-core:
	@echo "${GREEN}Running Rust core tests...${NC}"
	cargo test --lib

# Python tests
test-python:
	@echo "${GREEN}Running Python tests...${NC}"
	cd adapters/python && pytest -v --tb=short

# Go tests
test-go:
	@echo "${GREEN}Running Go tests...${NC}"
	cd adapters/go/gin && go test -v ./...
	cd adapters/go/fiber && go test -v ./...

# Java tests
test-java:
	@echo "${GREEN}Running Java tests...${NC}"
	cd adapters/java/springboot && mvn test
	cd adapters/java/quarkus && mvn test

# Rust adapter tests
test-rust:
	@echo "${GREEN}Running Rust adapter tests...${NC}"
	cd adapters/rust/actix && cargo test
	cd adapters/rust/rocket && cargo test

# .NET tests
test-dotnet:
	@echo "${GREEN}Running .NET tests...${NC}"
	cd adapters/dotnet && dotnet test

# Ruby tests
test-ruby:
	@echo "${GREEN}Running Ruby tests...${NC}"
	cd adapters/ruby/rails && ruby -Ilib:test test/x402_rails_test.rb

# PHP tests
test-php:
	@echo "${GREEN}Running PHP tests...${NC}"
	cd adapters/php/laravel && vendor/bin/phpunit
	cd adapters/php/symfony && vendor/bin/phpunit

# Docker commands
docker-build:
	@echo "${GREEN}Building Docker test images...${NC}"
	cd adapters && docker-compose build

docker-clean:
	@echo "${GREEN}Cleaning up Docker containers...${NC}"
	cd adapters && docker-compose down -v

# Quick test - runs core + Python (most common)
test-quick: test-core test-python
	@echo ""
	@echo "${GREEN}✓ Quick tests passed!${NC}"

# Coverage report (requires cargo-tarpaulin)
coverage:
	@echo "${GREEN}Running coverage report...${NC}"
	cargo tarpaulin --out html --open

# CI target (used by GitHub Actions)
ci: test-core
	@echo "${GREEN}CI tests passed!${NC}"
