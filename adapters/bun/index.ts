export interface BunX402Config {
  pricePerRequest: number;
  scheme: string;
  network: string;
  paymentToken: string;
  recipient: string;
  facilitatorUrl?: string;
}

export function createX402Middleware(config: BunX402Config) {
  return async (req: Request): Promise<Response | null> => {
    const paymentHeader = req.headers.get('X-Payment-Token');
    
    if (!paymentHeader) {
      return new Response(
        JSON.stringify({
          error: 'payment_required',
          price: config.pricePerRequest,
          currency: config.paymentToken,
          network: config.network,
          scheme: config.scheme,
          recipient: config.recipient,
        }),
        {
          status: 402,
          headers: {
            'Content-Type': 'application/json',
            'X-Payment-Required': 'true',
            'X-Price': config.pricePerRequest.toString(),
            'X-Currency': config.paymentToken,
            'X-Network': config.network,
          },
        }
      );
    }
    
    return null;
  };
}

export { BunX402Config as X402Config };

export async function startServer(config: BunX402Config) {
  const x402Middleware = createX402Middleware(config);
  
  const server = Bun.serve({
    port: 3000,
    async fetch(req) {
      const x402Response = await x402Middleware(req);
      if (x402Response) {
        return x402Response;
      }
      
      return new Response(JSON.stringify({ data: 'Premium content!' }), {
        headers: { 'Content-Type': 'application/json' },
      });
    },
  });
  
  console.log(`Server running on http://localhost:${server.port}`);
}
