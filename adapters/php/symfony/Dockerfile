# PHP Symfony Test Dockerfile
FROM php:8.2-cli

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libzip-dev \
    zip \
    unzip \
    && docker-php-ext-install zip

COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

RUN composer require --dev phpunit/phpunit:^10

COPY adapters/php/symfony/src /app/src
COPY adapters/php/symfony/tests /app/tests
COPY adapters/php/symfony/phpunit.xml /app/phpunit.xml

CMD ["vendor/bin/phpunit", "--colors=always"]
