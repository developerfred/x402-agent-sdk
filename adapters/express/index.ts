import express, { Request, Response, NextFunction, RequestHandler } from "express";
import {
  PaymentRequired,
  createSolanaPayment,
  createStacksPayment,
  isPaymentRequired,
  parsePaymentRequired,
} from "../../deno/types.ts";

export interface X402ExpressConfig {
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

export function x402Middleware(config: X402ExpressConfig): RequestHandler {
  return async (req: Request, res: Response, next: NextFunction) => {
    const paymentToken = req.headers["x-payment-token"];
    const paymentRequiredHeader = req.headers["x-payment-required"];

    if (!paymentToken && !paymentRequiredHeader) {
      const pr = createPaymentRequired(config);
      
      res.setHeader("X-Payment-Required", JSON.stringify(pr));
      return res.status(402).json({
        error: "Payment Required",
        payment_required: pr,
      });
    }

    if (paymentToken) {
      try {
        const token = typeof paymentToken === "string" 
          ? JSON.parse(paymentToken) 
          : paymentToken;
        
        const amount = parseInt(token.amount || "0", 10);
        if (amount < config.pricePerRequest) {
          return res.status(402).json({
            error: "Insufficient payment",
            required: config.pricePerRequest,
            received: amount,
          });
        }
        
        return next();
      } catch (e) {
        return res.status(400).json({
          error: "Invalid payment token",
        });
      }
    }

    next();
  };
}

function createPaymentRequired(config: X402ExpressConfig): PaymentRequired {
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

export function createX402Route(config: X402RouteConfig): X402RouteConfig {
  return config;
}

export function paymentMiddleware(
  routesConfig: Record<string, X402RouteConfig>
): RequestHandler {
  return (req: Request, res: Response, next: NextFunction) => {
    const routeKey = `${req.method} ${req.path}`;
    const routeConfig = routesConfig[routeKey];

    if (!routeConfig) {
      return next();
    }

    const paymentToken = req.headers["x-payment-token"];

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

      res.setHeader("X-Payment-Required", JSON.stringify(pr));
      return res.status(402).json({
        error: "Payment Required",
        payment_required: pr,
        accepted_schemes: routeConfig.accepts,
      });
    }

    next();
  };
}

export function isX402Enabled(): boolean {
  return process.env.X402_ENABLED !== "false";
}

export function getX402ResourceServer() {
  return {
    checkPayment: (headers: Record<string, string>) => {
      const token = headers["x-payment-token"];
      return !!token;
    },
  };
}

export { PaymentRequired };
