import Fastify from "fastify";
import { x402Plugin, isX402Enabled } from "./index.ts";

const fastify = Fastify({ logger: true });

console.log("x402 Enabled:", isX402Enabled());

// Register x402 middleware
await fastify.register(x402Plugin, {
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  description: "Fastify API Access"
});

// Routes
fastify.get("/", async (request, reply) => {
  return {
    message: "Welcome to x402 Fastify Example",
    x402_enabled: isX402Enabled()
  };
});

fastify.get("/api/data", async (request, reply) => {
  return {
    data: "This is paid premium data!",
    timestamp: new Date().toISOString()
  };
});

fastify.post("/api/analyze", async (request, reply) => {
  return {
    result: "Analysis complete!",
    input: request.body
  };
});

const start = async () => {
  try {
    await fastify.listen({ port: 3000 });
    console.log("x402 Fastify server running on http://localhost:3000");
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
