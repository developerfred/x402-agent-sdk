import type { Context, Next } from 'hono';

export interface HonoX402Config {
  pricePerRequest: number;
  scheme: string;
  network: string;
  paymentToken: string;
  recipient: string;
  facilitatorUrl?: string;
}

export function x402Middleware(config: HonoX402Config) {
  return async (c: Context, next: Next) => {
    const paymentHeader = c.req.header('X-Payment-Token');
    
    if (!paymentHeader) {
      c.status(402);
      c.header('X-Payment-Required', 'true');
      c.header('X-Price', config.pricePerRequest.toString());
      c.header('X-Currency', config.paymentToken);
      c.header('X-Network', config.network);
      
      return c.json({
        error: 'payment_required',
        price: config.pricePerRequest,
        currency: config.paymentToken,
        network: config.network,
        scheme: config.scheme,
        recipient: config.recipient,
      });
    }
    
    await next();
  };
}

export { HonoX402Config as X402Config };
