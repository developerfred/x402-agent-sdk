# Python FastAPI Test Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY adapters/python/requirements-test.txt /app/requirements-test.txt
RUN pip install --no-cache-dir -r requirements-test.txt

# Copy adapter code
COPY adapters/python/x402_fastapi.py /app/x402_fastapi.py
COPY adapters/python/test_x402_fastapi.py /app/test_x402_fastapi.py

# Run tests
CMD ["pytest", "-v", "--tb=short", "test_x402_fastapi.py"]
