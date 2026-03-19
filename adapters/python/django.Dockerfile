# Python Django Test Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Copy requirements and install dependencies
COPY adapters/python/requirements-test.txt /app/requirements-test.txt
RUN pip install --no-cache-dir -r requirements-test.txt

# Copy source files
COPY adapters/python/test_settings.py /app/test_settings.py
COPY adapters/python/x402_django.py /app/x402_django.py
COPY adapters/python/test_x402_django.py /app/test_x402_django.py

ENV DJANGO_SETTINGS_MODULE=test_settings

CMD ["pytest", "-v", "--tb=short", "test_x402_django.py"]
