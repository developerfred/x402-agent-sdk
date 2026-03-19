"""
x402 FastAPI Adapter Tests
=========================
"""

import pytest
import base64
import json
import time
from unittest.mock import Mock, patch, AsyncMock
from fastapi import FastAPI, Request
from fastapi.testclient import TestClient
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse

from x402_fastapi import (
    X402Middleware,
    X402Config,
    create_payment_required,
    parse_payment_token,
    ChainType,
)


class TestX402Config:
    """Tests for X402Config dataclass"""

    def test_default_config(self):
        config = X402Config(
            price_per_request=100,
            scheme="v1-eip155-exact",
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )
        assert config.price_per_request == 100
        assert config.scheme == "v1-eip155-exact"
        assert config.network == "eip155:1"
        assert config.expiry_seconds == 3600

    def test_custom_config(self):
        config = X402Config(
            price_per_request=500,
            scheme="v1-solana-exact",
            network="solana:101",
            payment_token="EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            recipient="SolanaAddress123",
            description="Premium API Access",
            expiry_seconds=7200,
            facilitator_url="https://custom-facilitator.example.com",
        )
        assert config.price_per_request == 500
        assert config.scheme == "v1-solana-exact"
        assert config.network == "solana:101"
        assert config.description == "Premium API Access"
        assert config.expiry_seconds == 7200


class TestCreatePaymentRequired:
    """Tests for create_payment_required function"""

    def test_create_evm_payment_required(self):
        config = X402Config(
            price_per_request=100,
            scheme="v1-eip155-exact",
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        pr = create_payment_required(config, "/api/data")

        assert pr["version"] == 1
        assert pr["scheme"] == "v1-eip155-exact"
        assert pr["network"] == "eip155:1"
        assert pr["payment_token"] == "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        assert pr["max_amount"] == "100"
        assert pr["recipient"] == "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
        assert pr["description"] == "API access"
        assert "expiry" in pr
        assert "salt" in pr
        assert "instructions" in pr

    def test_create_solana_payment_required(self):
        config = X402Config(
            price_per_request=50,
            scheme="v1-solana-exact",
            network="solana:101",
            payment_token="EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            recipient="SolanaRecipient123",
        )

        pr = create_payment_required(config, "/api/solana")

        assert pr["scheme"] == "v1-solana-exact"
        assert pr["network"] == "solana:101"
        assert pr["payment_token"] == "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ"

    def test_payment_required_has_valid_expiry(self):
        config = X402Config(
            price_per_request=100,
            scheme="v1-eip155-exact",
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            expiry_seconds=3600,
        )

        before = int(time.time())
        pr = create_payment_required(config, "/api/test")
        after = int(time.time())

        assert before + 3600 <= pr["expiry"] <= after + 3600

    def test_payment_required_instructions(self):
        config = X402Config(
            price_per_request=100,
            scheme="v1-eip155-exact",
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            facilitator_url="https://facilitator.x402.rs",
        )

        pr = create_payment_required(config, "/api/data")

        assert len(pr["instructions"]) == 1
        instruction = pr["instructions"][0]
        assert instruction["protocol"] == "https"
        assert instruction["method"] == "POST"
        assert instruction["url"] == "https://facilitator.x402.rs/v1/payment"
        assert "body" in instruction


class TestParsePaymentToken:
    """Tests for parse_payment_token function"""

    def test_parse_valid_token(self):
        token_data = {
            "signature": "0x1234",
            "sender": "0xabcd",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        mock_request = Mock(spec=Request)
        mock_request.headers = {"X-Payment-Token": encoded}

        result = parse_payment_token(mock_request)

        assert result is not None
        assert result["signature"] == "0x1234"
        assert result["sender"] == "0xabcd"

    def test_parse_no_token(self):
        mock_request = Mock(spec=Request)
        mock_request.headers = {}

        result = parse_payment_token(mock_request)

        assert result is None

    def test_parse_invalid_token(self):
        mock_request = Mock(spec=Request)
        mock_request.headers = {"X-Payment-Token": "invalid-base64!!!="}

        result = parse_payment_token(mock_request)

        assert result is None


class TestX402Middleware:
    """Tests for X402Middleware"""

    @pytest.fixture
    def app(self):
        app = FastAPI()

        config = X402Config(
            price_per_request=100,
            scheme="v1-eip155-exact",
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        )

        app.add_middleware(X402Middleware, config=config)

        @app.get("/api/data")
        async def get_data():
            return {"data": "success"}

        return app

    @pytest.fixture
    def client(self, app):
        return TestClient(app)

    def test_request_without_payment_returns_402(self, client):
        response = client.get("/api/data")

        assert response.status_code == 402
        data = response.json()
        assert "payment_token" in data
        assert "expiry" in data

    def test_request_with_valid_token_succeeds(self, client):
        token_data = {
            "signature": "0xabc123",
            "sender": "0x sender",
            "amount": "100",
            "valid_after": int(time.time()),
        }
        encoded = base64.b64encode(json.dumps(token_data).encode()).decode()

        response = client.get("/api/data", headers={"X-Payment-Token": encoded})

        assert response.status_code == 200

    def test_excluded_path_no_payment_required(self, client):
        response = client.get("/docs")
        assert response.status_code == 200


class TestChainType:
    """Tests for ChainType enum"""

    def test_chain_types(self):
        assert ChainType.EVM.value == "evm"
        assert ChainType.SOLANA.value == "solana"
        assert ChainType.STACKS.value == "stacks"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
