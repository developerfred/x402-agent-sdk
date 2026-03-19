# PHP Laravel Test Dockerfile
FROM php:8.2-cli

WORKDIR /app

# Install extensions and dependencies
RUN apt-get update && apt-get install -y \
    libzip-dev \
    zip \
    unzip \
    && docker-php-ext-install zip

# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Install PHP dependencies
RUN composer require --dev phpunit/phpunit:^10

# Copy source
COPY adapters/php/laravel/app /app/app
COPY adapters/php/laravel/tests /app/tests
COPY adapters/php/laravel/phpunit.xml /app/phpunit.xml

CMD ["vendor/bin/phpunit", "--colors=always"]
