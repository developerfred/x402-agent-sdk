import { NextRequest, NextResponse } from "next/server";
import { PaymentRequired } from "../../deno/types.ts";

export interface X402NextJsConfig {
  pricePerRequest: number;
  scheme: "v1-solana-exact" | "v2-stacks-exact" | "v1-eip155-exact";
  network: string;
  paymentToken: string;
  recipient: string;
  description?: string;
}

export interface X402RouteConfig {
  accepts: string[];
  description?: string;
  price?: number;
}

export function createPaymentRequired(config: X402NextJsConfig): PaymentRequired {
  return {
    version: 1,
    scheme: config.scheme,
    network: config.network,
    payment_token: config.paymentToken,
    max_amount: config.pricePerRequest.toString(),
    recipient: config.recipient,
    description: config.description,
  };
}

export function x402Middleware(config: X402NextJsConfig) {
  return async (req: NextRequest) => {
    const paymentToken = req.headers.get("x-payment-token");
    const paymentRequiredHeader = req.headers.get("x-payment-required");

    if (!paymentToken && !paymentRequiredHeader) {
      const pr = createPaymentRequired(config);

      return NextResponse.json(
        {
          error: "Payment Required",
          payment_required: pr,
        },
        {
          status: 402,
          headers: {
            "X-Payment-Required": JSON.stringify(pr),
          },
        }
      );
    }

    if (paymentToken) {
      try {
        const token = JSON.parse(paymentToken);
        
        const amount = parseInt(token.amount || "0", 10);
        if (amount < config.pricePerRequest) {
          return NextResponse.json(
            {
              error: "Insufficient payment",
              required: config.pricePerRequest,
              received: amount,
            },
            { status: 402 }
          );
        }
        
        return null;
      } catch (e) {
        return NextResponse.json(
          { error: "Invalid payment token" },
          { status: 400 }
        );
      }
    }

    return null;
  };
}

export function paymentMiddleware(
  routesConfig: Record<string, X402RouteConfig>
) {
  return async (req: NextRequest) => {
    const url = new URL(req.url);
    const routeKey = `${req.method} ${url.pathname}`;
    const routeConfig = routesConfig[routeKey];

    if (!routeConfig) {
      return null;
    }

    const paymentToken = req.headers.get("x-payment-token");

    if (!paymentToken) {
      const pr: PaymentRequired = {
        version: 1,
        scheme: "v1-eip155-exact",
        network: "eip155:1",
        payment_token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        max_amount: (routeConfig.price || 100).toString(),
        recipient: "0x0000000000000000000000000000000000000000",
        description: routeConfig.description,
      };

      return NextResponse.json(
        {
          error: "Payment Required",
          payment_required: pr,
          accepted_schemes: routeConfig.accepts,
        },
        {
          status: 402,
          headers: {
            "X-Payment-Required": JSON.stringify(pr),
          },
        }
      );
    }

    return null;
  };
}

export function isX402Enabled(): boolean {
  return process.env.X402_ENABLED !== "false";
}

export function getX402Config() {
  return {
    network: process.env.X402_NETWORK || "eip155:1",
    paymentToken: process.env.X402_PAYMENT_TOKEN || "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    recipient: process.env.X402_RECIPIENT || "0x0000000000000000000000000000000000000000",
    facilitatorUrl: process.env.X402_FACILITATOR_URL || "https://facilitator.x402.rs",
  };
}

export { PaymentRequired };
