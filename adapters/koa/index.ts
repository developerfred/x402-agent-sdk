import type { Context, Next } from 'koa';

export interface KoaX402Config {
  pricePerRequest: number;
  scheme: string;
  network: string;
  paymentToken: string;
  recipient: string;
  facilitatorUrl?: string;
  description?: string;
}

export function x402Middleware(config: KoaX402Config) {
  return async (ctx: Context, next: Next) => {
    const paymentHeader = ctx.get('X-Payment-Token');
    
    if (!paymentHeader) {
      ctx.status = 402;
      ctx.set('X-Payment-Required', 'true');
      ctx.set('X-Price', config.pricePerRequest.toString());
      ctx.set('X-Currency', config.paymentToken);
      ctx.set('X-Network', config.network);
      
      ctx.body = {
        error: 'payment_required',
        price: config.pricePerRequest,
        currency: config.paymentToken,
        network: config.network,
        scheme: config.scheme,
        recipient: config.recipient,
      };
      return;
    }
    
    await next();
  };
}

export { KoaX402Config as X402Config };
