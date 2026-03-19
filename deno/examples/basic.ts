import { X402Client, X402Server, createSolanaPayment, createStacksPayment } from "./mod.ts";

const server = new X402Server(
  100,
  "v1-solana-exact",
  "solana:101",
  "USDC",
  "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
);

const pr = server.createPaymentRequired("API Access");
console.log("Payment Required:", pr);

const client = new X402Client({
  facilitatorUrl: "https://facilitator.x402.rs",
  maxRetries: 3,
});

console.log("Client initialized");

const solanaPayment = createSolanaPayment(
  "mainnet",
  "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
  "USDC",
  100,
);
console.log("Solana Payment:", solanaPayment);

const stacksPayment = createStacksPayment(
  "mainnet",
  "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
  "STX",
  100,
);
console.log("Stacks Payment:", stacksPayment);

console.log("Example completed!");
