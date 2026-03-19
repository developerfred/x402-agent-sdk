import Fastify, { FastifyRequest, FastifyReply, FastifyInstance } from "fastify";
import { PaymentRequired } from "../../deno/types.ts";

export interface X402FastifyConfig {
  pricePerRequest: number;
  scheme: "v1-solana-exact" | "v2-stacks-exact" | "v1-eip155-exact";
  network: string;
  paymentToken: string;
  recipient: string;
  description?: string;
}

export interface X402FastifyRouteConfig {
  accepts: string[];
  description?: string;
  price?: number;
}

declare module "fastify" {
  interface FastifyRequest {
    x402PaymentVerified?: boolean;
    x402PaymentToken?: object;
  }
}

export async function x402Plugin(
  fastify: FastifyInstance,
  options: X402FastifyConfig
) {
  fastify.addHook("onRequest", async (request: FastifyRequest, reply: FastifyReply) => {
    const paymentToken = request.headers["x-payment-token"];
    const paymentRequiredHeader = request.headers["x-payment-required"];

    if (!paymentToken && !paymentRequiredHeader) {
      const pr = createPaymentRequired(options);

      reply.code(402);
      reply.header("X-Payment-Required", JSON.stringify(pr));
      
      return reply.send({
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
        if (amount < options.pricePerRequest) {
          reply.code(402);
          return reply.send({
            error: "Insufficient payment",
            required: options.pricePerRequest,
            received: amount,
          });
        }
        
        request.x402PaymentVerified = true;
        request.x402PaymentToken = token;
      } catch (e) {
        reply.code(400);
        return reply.send({
          error: "Invalid payment token",
        });
      }
    }
  });
}

function createPaymentRequired(config: X402FastifyConfig): PaymentRequired {
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

export function createX402RouteConfig(config: X402FastifyRouteConfig): X402FastifyRouteConfig {
  return config;
}

export function paymentMiddleware(
  routesConfig: Record<string, X402FastifyRouteConfig>
) {
  return async (request: FastifyRequest, reply: FastifyReply) => {
    const routeKey = `${request.method} ${request.url}`;
    const routeConfig = routesConfig[routeKey];

    if (!routeConfig) {
      return;
    }

    const paymentToken = request.headers["x-payment-token"];

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

      reply.code(402);
      reply.header("X-Payment-Required", JSON.stringify(pr));
      
      return reply.send({
        error: "Payment Required",
        payment_required: pr,
        accepted_schemes: routeConfig.accepts,
      });
    }
  };
}

export function isX402Enabled(): boolean {
  return process.env.X402_ENABLED !== "false";
}
