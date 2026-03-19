"""
x402 Flask Adapter Tests
========================
"""

import pytest
import base64
import json
import time
from unittest.mock import Mock, patch, MagicMock
from flask import Flask, jsonify

from x402_flask import (
    X402,
    X402Config,
    create_payment_required,
    parse_payment_token,
)


class TestX402Config:
    """Tests for X402Config"""

    def test_default_config(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )
        assert config.price_per_request == 100
        assert config.scheme == "v1-eip155-exact"
        assert config.network == "eip155:1"

    def test_custom_config(self):
        config = X402Config(
            price_per_request=250,
            scheme="v1-solana-exact",
            network="solana:101",
            payment_token="EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            recipient="SolanaAddr123",
            description="Premium Access",
            expiry_seconds=1800,
        )
        assert config.price_per_request == 250
        assert config.description == "Premium Access"


class TestCreatePaymentRequiredFlask:
    """Tests for create_payment_required in Flask context"""

    def test_create_evm_payment(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        pr = create_payment_required(config, "/api/endpoint")

        assert pr["version"] == 1
        assert pr["scheme"] == "v1-eip155-exact"
        assert pr["network"] == "eip155:1"
        assert pr["max_amount"] == "100"

    def test_create_solana_payment(self):
        config = X402Config(
            price_per_request=50,
            scheme="v1-solana-exact",
            network="solana:101",
            payment_token="EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            recipient="SolanaRecipient",
        )

        pr = create_payment_required(config, "/api/solana")

        assert pr["scheme"] == "v1-solana-exact"
        assert pr["network"] == "solana:101"

    def test_payment_expiry(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xABC",
            recipient="0x123",
            expiry_seconds=7200,
        )

        before = int(time.time())
        pr = create_payment_required(config, "/test")
        after = int(time.time())

        assert before + 7200 <= pr["expiry"] <= after + 7200


class TestParsePaymentTokenFlask:
    """Tests for parse_payment_token in Flask context"""

    def test_parse_valid_token(self):
        token_data = {
            "signature": "sig123",
            "sender": "sender456",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        mock_request = Mock()
        mock_request.headers = {"X-Payment-Token": encoded}

        result = parse_payment_token(mock_request)

        assert result is not None
        assert result["signature"] == "sig123"

    def test_parse_missing_token(self):
        mock_request = Mock()
        mock_request.headers = {}

        result = parse_payment_token(mock_request)

        assert result is None

    def test_parse_invalid_base64(self):
        mock_request = Mock()
        mock_request.headers = {"X-Payment-Token": "not-valid-base64!!"}

        result = parse_payment_token(mock_request)

        assert result is None


class TestX402Extension:
    """Tests for X402 Flask extension"""

    @pytest.fixture
    def app(self):
        app = Flask(__name__)
        app.config["TESTING"] = True

        config = X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        x402 = X402(app, config=config)

        @app.route("/api/data")
        def get_data():
            return jsonify({"data": "success"})

        return app

    @pytest.fixture
    def client(self, app):
        return app.test_client()

    def test_no_payment_returns_402(self, client):
        response = client.get("/api/data")

        assert response.status_code == 402
        data = response.get_json()
        assert "payment_token" in data

    def test_with_valid_token_succeeds(self, client):
        token_data = {
            "signature": "valid_sig",
            "sender": "valid_sender",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        response = client.get("/api/data", headers={"X-Payment-Token": encoded})

        assert response.status_code == 200


class TestX402Middleware:
    """Tests for x402_middleware convenience function"""

    def test_middleware_creation(self):
        from x402_flask import x402_middleware

        app = Flask(__name__)

        middleware = x402_middleware(
            app, price_per_request=100, payment_token="0xABC", recipient="0x123"
        )

        assert middleware is not None
        assert isinstance(middleware, X402)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
