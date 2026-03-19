# Python Flask Test Dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY adapters/python/requirements-test.txt /app/requirements-test.txt
RUN pip install --no-cache-dir -r requirements-test.txt

COPY adapters/python/x402_flask.py /app/x402_flask.py
COPY adapters/python/test_x402_flask.py /app/test_x402_flask.py

CMD ["pytest", "-v", "--tb=short", "test_x402_flask.py"]
