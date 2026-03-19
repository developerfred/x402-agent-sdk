import express from "express";
import { x402Middleware, paymentMiddleware, isX402Enabled } from "./index.ts";

const app = express();
app.use(express.json());

console.log("x402 Enabled:", isX402Enabled());

// Example 1: Simple global payment requirement
app.use(x402Middleware({
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  description: "General API Access"
}));

// Example 2: Route-specific payment with different prices
// app.use(paymentMiddleware({
//   "GET /api/free": {
//     accepts: [],
//     description: "Free endpoint",
//     price: 0
//   },
//   "GET /api/premium": {
//     accepts: ["v1-eip155-exact", "v1-solana-exact"],
//     description: "Premium data",
//     price: 500
//   },
//   "POST /api/analyze": {
//     accepts: ["v1-eip155-exact"],
//     description: "AI Analysis - higher price",
//     price: 1000
//   }
// }));

// Routes
app.get("/", (req, res) => {
  res.json({ 
    message: "Welcome to x402 Express Example",
    x402_enabled: isX402Enabled()
  });
});

app.get("/api/data", (req, res) => {
  res.json({
    data: "This is paid premium data!",
    timestamp: new Date().toISOString()
  });
});

app.post("/api/analyze", (req, res) => {
  res.json({
    result: "Analysis complete!",
    input: req.body
  });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`x402 Express server running on http://localhost:${PORT}`);
  console.log("Try accessing /api/data to see the payment flow in action!");
});
