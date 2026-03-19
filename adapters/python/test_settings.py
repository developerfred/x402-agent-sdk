# Django settings for tests
SECRET_KEY = "test-secret-key"
DEBUG = True
ALLOWED_HOSTS = ["*"]

X402_CONFIG = {
    "price_per_request": 100,
    "scheme": "v1-eip155-exact",
    "network": "eip155:1",
    "payment_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "recipient": "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
}
