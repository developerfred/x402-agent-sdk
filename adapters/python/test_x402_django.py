"""
x402 Django Adapter Tests
=========================
"""

import pytest
import base64
import json
import time
from unittest.mock import Mock, patch, MagicMock
from django.test import TestCase, RequestFactory, override_settings
from django.http import HttpResponse

try:
    from x402_django import (
        X402Middleware,
        X402Config,
        create_payment_required,
        parse_payment_token,
        get_config,
    )

    DJANGO_AVAILABLE = True
except ImportError:
    DJANGO_AVAILABLE = False


pytestmark = pytest.mark.skipif(not DJANGO_AVAILABLE, reason="Django not available")


class TestX402ConfigDjango:
    """Tests for X402Config"""

    def test_config_creation(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        assert config.price_per_request == 100
        assert config.network == "eip155:1"

    def test_custom_config(self):
        config = X402Config(
            price_per_request=500,
            scheme="v1-solana-exact",
            network="solana:101",
            payment_token="EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            recipient="SolanaAddr",
            description="Premium",
            expiry_seconds=1800,
        )

        assert config.price_per_request == 500
        assert config.scheme == "v1-solana-exact"


class TestCreatePaymentRequiredDjango:
    """Tests for create_payment_required"""

    def test_create_evm_payment(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        pr = create_payment_required(config, "/api/test")

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
            recipient="SolanaRec",
        )

        pr = create_payment_required(config, "/api/sol")

        assert pr["scheme"] == "v1-solana-exact"
        assert pr["network"] == "solana:101"

    def test_expiry_timestamp(self):
        config = X402Config(
            price_per_request=100,
            payment_token="0xABC",
            recipient="0x123",
            expiry_seconds=3600,
        )

        before = int(time.time())
        pr = create_payment_required(config, "/test")
        after = int(time.time())

        assert before + 3600 <= pr["expiry"] <= after + 3600


class TestParsePaymentTokenDjango:
    """Tests for parse_payment_token"""

    def test_parse_valid_token(self):
        token_data = {
            "signature": "sig123",
            "sender": "sender456",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        mock_request = Mock()
        mock_request.META = {"HTTP_X_PAYMENT_TOKEN": encoded}

        result = parse_payment_token(mock_request)

        assert result is not None
        assert result["signature"] == "sig123"

    def test_parse_missing_token(self):
        mock_request = Mock()
        mock_request.META = {}

        result = parse_payment_token(mock_request)

        assert result is None


class TestX402Middleware:
    """Tests for X402Middleware"""

    @pytest.fixture
    def factory(self):
        return RequestFactory()

    @pytest.fixture
    def config(self):
        return X402Config(
            price_per_request=100,
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

    def test_middleware_initialization(self, config):
        get_response = Mock(return_value=HttpResponse("OK"))
        middleware = X402Middleware(get_response)

        assert middleware.config == config

    def test_excluded_paths(self, factory, config):
        get_response = Mock(return_value=HttpResponse("OK"))
        middleware = X402Middleware(get_response)

        request = factory.get("/admin/dashboard")
        middleware(request)

        get_response.assert_called_once()

    def test_no_token_returns_402(self, factory, config):
        get_response = Mock(return_value=HttpResponse("OK"))
        middleware = X402Middleware(get_response)

        request = factory.get("/api/data")
        response = middleware(request)

        assert response.status_code == 402

    def test_valid_token_passes(self, factory, config):
        get_response = Mock(return_value=HttpResponse("OK"))
        middleware = X402Middleware(get_response)

        token_data = {
            "signature": "valid",
            "sender": "sender",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        request = factory.get("/api/data", HTTP_X_PAYMENT_TOKEN=encoded)
        response = middleware(request)

        assert response.status_code == 200


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
