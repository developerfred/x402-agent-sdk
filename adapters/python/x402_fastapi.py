"""
x402 FastAPI Adapter
====================
Middleware for adding x402 payment capabilities to FastAPI applications.

Usage:
    from x402_fastapi import X402Middleware, X402Config

    config = X402Config(
        price_per_request=100,
        scheme="v1-eip155-exact",
        network="eip155:1",
        payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
    )

    app = FastAPI()
    app.add_middleware(X402Middleware, config=config)
"""

from typing import Optional, Dict, Any
from dataclasses import dataclass
from enum import Enum
import hashlib
import time

from fastapi import Request, Response, HTTPException
from fastapi.responses import JSONResponse
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp


class ChainType(str, Enum):
    """Supported blockchain chains"""

    EVM = "evm"
    SOLANA = "solana"
    STACKS = "stacks"


@dataclass
class X402Config:
    """Configuration for x402 payments"""

    price_per_request: int
    payment_token: str
    recipient: str
    scheme: str = "v1-eip155-exact"  # e.g., "v1-eip155-exact"
    network: str = "eip155:1"  # e.g., "eip155:1" for Ethereum mainnet
    description: Optional[str] = "API access"
    expiry_seconds: Optional[int] = 3600
    facilitator_url: Optional[str] = "https://facilitator.x402.rs"


class PaymentRequired(JSONResponse):
    """HTTP 402 Payment Required response"""

    def __init__(self, content: Dict[str, Any], status_code: int = 402, **kwargs):
        super().__init__(content=content, status_code=status_code, **kwargs)


def create_payment_required(config: X402Config, path: str = "/") -> Dict[str, Any]:
    """Create a PaymentRequired object according to x402 spec"""
    now = int(time.time())
    expiry = now + (config.expiry_seconds or 3600)

    # Generate a salt for this specific payment request
    salt = hashlib.sha256(f"{path}:{now}".encode()).hexdigest()[:16]

    return {
        "version": 1,
        "scheme": config.scheme,
        "network": config.network,
        "payment_token": config.payment_token,
        "max_amount": str(config.price_per_request),
        "recipient": config.recipient,
        "description": config.description,
        "expiry": expiry,
        "salt": salt,
        "instructions": [
            {
                "protocol": "https",
                "method": "POST",
                "url": f"{config.facilitator_url}/v1/payment",
                "headers": {"Content-Type": "application/json"},
                "body": {
                    "network": config.network,
                    "payment_token": config.payment_token,
                    "amount": str(config.price_per_request),
                    "recipient": config.recipient,
                    "valid_after": now,
                    "salt": salt,
                },
            }
        ],
    }


def parse_payment_token(request: Request) -> Optional[Dict[str, Any]]:
    """Parse X-Payment-Token header from request"""
    token_header = request.headers.get("X-Payment-Token")

    if not token_header:
        return None

    try:
        # Token format: base64(JSON)
        import base64

        decoded = base64.b64decode(token_header).decode()
        import json

        return json.loads(decoded)
    except Exception:
        return None


class X402Middleware(BaseHTTPMiddleware):
    """
    FastAPI middleware for x402 payment processing.

    This middleware intercepts requests and checks for valid payment tokens.
    If no valid token is present, it returns a 402 Payment Required response.
    """

    def __init__(
        self,
        app: ASGIApp,
        config: X402Config,
        exclude_paths: Optional[list] = None,
        skip_payment_check: Optional[callable] = None,
    ):
        super().__init__(app)
        self.config = config
        self.exclude_paths = exclude_paths or ["/docs", "/openapi.json", "/redoc"]
        self.skip_payment_check = skip_payment_check

    async def dispatch(self, request: Request, call_next):
        # Skip payment check for excluded paths
        if request.url.path in self.exclude_paths:
            return await call_next(request)

        # Custom skip function
        if self.skip_payment_check and await self._is_async_callable(
            self.skip_payment_check
        ):
            if await self.skip_payment_check(request):
                return await call_next(request)
        elif self.skip_payment_check and self.skip_payment_check(request):
            return await call_next(request)

        # Check for valid payment token
        payment_token = parse_payment_token(request)

        if payment_token:
            # Verify token validity (simplified - in production, verify signature)
            if self._verify_payment_token(payment_token):
                # Token valid - proceed with request
                return await call_next(request)

        # No valid token - return 402 Payment Required
        payment_required = create_payment_required(self.config, request.url.path)

        return PaymentRequired(content=payment_required)

    def _verify_payment_token(self, token: Dict[str, Any]) -> bool:
        """Verify payment token - simplified version"""
        # In production, verify:
        # 1. Token is not expired
        # 2. Token signature is valid
        # 3. Token matches the payment request (salt, amount, etc.)
        required_fields = ["signature", "sender", "amount", "valid_after"]
        return all(field in token for field in required_fields)

    async def _is_async_callable(self, obj):
        import asyncio

        return asyncio.iscoroutinefunction(obj)


# Convenience function for easy setup
def x402_middleware(
    app: ASGIApp,
    *,
    price_per_request: int,
    scheme: str = "v1-eip155-exact",
    network: str = "eip155:1",
    payment_token: str,
    recipient: str,
    description: str = "API access",
    exclude_paths: Optional[list] = None,
) -> X402Middleware:
    """
    Add x402 payment middleware to a FastAPI application.

    Example:
        from fastapi import FastAPI
        from x402_fastapi import x402_middleware

        app = FastAPI()
        x402_middleware(
            app,
            price_per_request=100,
            network="eip155:1",
            payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
        )
    """
    config = X402Config(
        price_per_request=price_per_request,
        scheme=scheme,
        network=network,
        payment_token=payment_token,
        recipient=recipient,
        description=description,
    )

    return X402Middleware(app, config=config, exclude_paths=exclude_paths)


__all__ = [
    "X402Middleware",
    "X402Config",
    "PaymentRequired",
    "create_payment_required",
    "parse_payment_token",
    "x402_middleware",
    "ChainType",
]
