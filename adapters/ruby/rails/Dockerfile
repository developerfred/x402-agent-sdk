# Ruby Rails Test Dockerfile
FROM ruby:3.2-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install bundler and gems
RUN gem install minitest rake

# Copy test files
COPY adapters/ruby/rails/lib /app/lib
COPY adapters/ruby/rails/test /app/test

CMD ["ruby", "-Ilib:test", "test/x402_rails_test.rb"]
