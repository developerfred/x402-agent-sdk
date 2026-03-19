"""
x402 Django Middleware
=====================
Middleware for adding x402 payment capabilities to Django applications.

Usage:
    # settings.py
    MIDDLEWARE = [
        ...
        'x402_django.X402Middleware',
    ]

    X402_CONFIG = {
        'price_per_request': 100,
        'network': 'eip155:1',
        'payment_token': '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
        'recipient': '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
    }
"""

from typing import Optional, Dict, Any
from dataclasses import dataclass
import hashlib
import time
import base64
import json

from django.http import HttpRequest, JsonResponse, HttpResponse


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


def get_config() -> X402Config:
    """Get X402 config from Django settings"""
    from django.conf import settings

    config_dict = getattr(settings, "X402_CONFIG", {})

    return X402Config(
        price_per_request=config_dict.get("price_per_request", 100),
        scheme=config_dict.get("scheme", "v1-eip155-exact"),
        network=config_dict.get("network", "eip155:1"),
        payment_token=config_dict.get("payment_token", ""),
        recipient=config_dict.get("recipient", ""),
        description=config_dict.get("description", "API access"),
        expiry_seconds=config_dict.get("expiry_seconds", 3600),
        facilitator_url=config_dict.get(
            "facilitator_url", "https://facilitator.x402.rs"
        ),
    )


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


def parse_payment_token(request: HttpRequest) -> Optional[Dict[str, Any]]:
    """Parse X-Payment-Token header from request"""
    token_header = request.META.get("HTTP_X_PAYMENT_TOKEN")

    if not token_header:
        return None

    try:
        decoded = base64.b64decode(token_header).decode()
        return json.loads(decoded)
    except Exception:
        return None


class X402Middleware:
    """
    Django middleware for x402 payment processing.

    Add to MIDDLEWARE in settings.py:
        MIDDLEWARE = [
            ...
            'x402_django.X402Middleware',
        ]

    Configure in settings.py:
        X402_CONFIG = {
            'price_per_request': 100,
            'network': 'eip155:1',
            'payment_token': '0x...',
            'recipient': '0x...',
        }
    """

    def __init__(self, get_response):
        self.get_response = get_response
        self.config = get_config()
        self.exclude_paths = ["/admin/", "/static/", "/media/"]

    def __call__(self, request: HttpRequest):
        # Skip excluded paths
        if any(request.path.startswith(path) for path in self.exclude_paths):
            return self.get_response(request)

        # Check for valid payment token
        payment_token = parse_payment_token(request)

        if payment_token and self._verify_payment_token(payment_token):
            # Token valid - proceed
            return self.get_response(request)

        # No valid token - return 402 Payment Required
        response = JsonResponse(
            create_payment_required(self.config, request.path), status=402
        )
        return response

    def _verify_payment_token(self, token: Dict[str, Any]) -> bool:
        """Verify payment token"""
        required_fields = ["signature", "sender", "amount", "valid_after"]
        return all(field in token for field in required_fields)


def require_payment(view_func):
    """
    Decorator to explicitly require payment for a view.

    Usage:
        @require_payment
        def my_view(request):
            return JsonResponse({"data": "paid content"})
    """

    def wrapped(request: HttpRequest, *args, **kwargs):
        payment_token = parse_payment_token(request)

        if not payment_token or not X402Middleware(
            lambda r: None
        )._verify_payment_token(payment_token):
            config = get_config()
            return JsonResponse(
                create_payment_required(config, request.path), status=402
            )

        return view_func(request, *args, **kwargs)

    return wrapped


def x402_response(request: HttpRequest) -> JsonResponse:
    """Generate a 402 Payment Required response"""
    config = get_config()
    return JsonResponse(create_payment_required(config, request.path), status=402)


__all__ = [
    "X402Middleware",
    "X402Config",
    "require_payment",
    "x402_response",
    "get_config",
]
