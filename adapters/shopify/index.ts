/**
 * Shopify x402 App - Payment-enabled Shopify store
 * 
 * Add x402 payments to any Shopify store
 */

export interface ShopifyX402Config {
  network: string;
  paymentToken: string;
  recipient: string;
  facilitatorUrl?: string;
  products: Record<string, number>;
}

export interface ShopifyWebhookPayload {
  topic: string;
  shop_domain: string;
  body: any;
}

export function createShopifyX402Middleware(config: ShopifyX402Config) {
  return async (req: Request): Promise<Response | null> => {
    const url = new URL(req.url);
    
    if (!url.pathname.startsWith('/products/')) {
      return null;
    }

    const productHandle = url.pathname.split('/products/')[1];
    const price = config.products[productHandle];

    if (!price) {
      return new Response('Product not found', { status: 404 });
    }

    const paymentHeader = req.headers.get('X-Payment-Token');
    
    if (!paymentHeader) {
      return new Response(
        JSON.stringify({
          error: 'payment_required',
          price,
          currency: config.paymentToken,
          network: config.network,
        }),
        {
          status: 402,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    return null;
  };
}

export async function handleShopifyWebhook(
  payload: ShopifyWebhookPayload,
  config: ShopifyX402Config
): Promise<{ success: boolean; message: string }> {
  switch (payload.topic) {
    case 'orders/create':
      return handleOrderCreated(payload.body, config);
    case 'refunds/create':
      return handleRefundCreated(payload.body, config);
    default:
      return { success: true, message: 'Unhandled webhook' };
  }
}

async function handleOrderCreated(order: any, config: ShopifyX402Config) {
  const totalPrice = parseFloat(order.total_price);
  const currency = order.currency;
  
  console.log(`Order created: ${order.id}, Total: ${totalPrice} ${currency}`);
  
  return { success: true, message: 'Order processed' };
}

async function handleRefundCreated(refund: any, config: ShopifyX402Config) {
  console.log(`Refund created: ${refund.id}`);
  
  return { success: true, message: 'Refund processed' };
}

export function verifyShopifyWebhook(
  body: string,
  hmacHeader: string,
  secret: string
): boolean {
  const crypto = require('crypto');
  const hash = crypto
    .createHmac('sha256', secret)
    .update(body, 'utf8')
    .digest('base64');
  
  return hash === hmacHeader;
}
