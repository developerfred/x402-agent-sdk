"""
x402 Flask Adapter
=================
Extension for adding x402 payment capabilities to Flask applications.

Usage:
    from x402_flask import X402, X402Config

    config = X402Config(
        price_per_request=100,
        network="eip155:1",
        payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
    )

    x402 = X402(app, config=config)
"""

from typing import Optional, Dict, Any, Callable
from dataclasses import dataclass
import hashlib
import time
import base64
import json

from flask import Flask, Request, Response, jsonify, request, g


@dataclass
class X402Config:
    """Configuration for x402 payments"""

    price_per_request: int
    payment_token: str
    recipient: str
    scheme: str = "v1-eip155-exact"
    network: str = "eip155:1"
    description: str = "API access"
    expiry_seconds: int = 3600
    facilitator_url: str = "https://facilitator.x402.rs"


def create_payment_required(config: X402Config, path: str = "/") -> Dict[str, Any]:
    """Create a PaymentRequired object according to x402 spec"""
    now = int(time.time())
    expiry = now + config.expiry_seconds
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
        decoded = base64.b64decode(token_header).decode()
        return json.loads(decoded)
    except Exception:
        return None


class X402:
    """
    Flask extension for x402 payment processing.

    Usage:
        x402 = X402(app, config=config)

        @app.route('/protected')
        def protected():
            return jsonify({"data": "paid content"})
    """

    def __init__(
        self,
        app: Optional[Flask] = None,
        config: Optional[X402Config] = None,
        exclude_urls: Optional[list] = None,
    ):
        self.config = config
        self.exclude_urls = exclude_urls or ["/"]
        self._app = app

        if app is not None:
            self.init_app(app)

    def init_app(self, app: Flask):
        """Initialize the extension with a Flask app"""
        self._app = app

        if not self.config:
            self.config = app.config.get("X402_CONFIG")

        app.before_request(self._check_payment)

        app.add_url_rule(
            "/x402/payment-required",
            "_x402_payment_required",
            self._payment_required_handler,
            methods=["GET"],
        )

    def _check_payment(self):
        """Check for valid payment token before request"""
        from flask import request, g

        if request.path in self.exclude_urls:
            return None

        if hasattr(self, "_skip_payment_check"):
            if self._skip_payment_check():
                return None

        payment_token = parse_payment_token(request)

        if payment_token and self._verify_payment_token(payment_token):
            g.x402_payment_valid = True
            return None

        g.x402_payment_valid = False
        return self._create_402_response()

    def _create_402_response(self):
        """Create 402 Payment Required response"""
        response = jsonify(create_payment_required(self.config, request.path))
        response.status_code = 402
        return response

    def _payment_required_handler(self):
        """Handler for payment-required endpoint"""
        return jsonify(
            create_payment_required(self.config, request.args.get("path", "/"))
        )

    def _verify_payment_token(self, token: Dict[str, Any]) -> bool:
        """Verify payment token"""
        required_fields = ["signature", "sender", "amount", "valid_after"]
        return all(field in token for field in required_fields)

    def skip_payment(self, f: Callable):
        """Decorator to skip payment check for a route"""
        self._skip_payment_check = f
        return f

    def require_payment(self, f: Callable):
        """Decorator to explicitly require payment for a route"""

        @functools.wraps(f)
        def decorated(*args, **kwargs):
            from flask import g

            if not getattr(g, "x402_payment_valid", False):
                return self._create_402_response()
            return f(*args, **kwargs)

        return decorated


def x402_middleware(
    app: Flask,
    *,
    price_per_request: int,
    scheme: str = "v1-eip155-exact",
    network: str = "eip155:1",
    payment_token: str,
    recipient: str,
    description: str = "API access",
    exclude_urls: Optional[list] = None,
) -> X402:
    """
    Add x402 payment middleware to a Flask application.

    Example:
        from flask import Flask
        from x402_flask import x402_middleware

        app = Flask(__name__)
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

    return X402(app, config=config, exclude_urls=exclude_urls)


import functools

__all__ = ["X402", "X402Config", "create_payment_required", "x402_middleware"]
