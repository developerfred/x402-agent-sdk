/**
 * WordPress x402 Plugin
 * 
 * Add x402 payments to WordPress sites
 */

export interface WordPressX402Config {
  network: string;
  paymentToken: string;
  recipient: string;
  facilitatorUrl?: string;
  protectedPosts: number[];
  protectedCategories: number[];
}

export interface WordPressRequest {
  url: string;
  method: string;
  headers: Map<string, string>;
  body?: string;
}

export function createWordPressX402Middleware(config: WordPressX402Config) {
  return async (req: WordPressRequest): Promise<Response | null> => {
    const url = new URL(req.url);
    
    const isProtectedPost = config.protectedPosts.some(id => 
      url.pathname.includes(`/posts/${id}`)
    );
    
    const isProtectedCategory = config.protectedCategories.some(id =>
      url.pathname.includes(`/category/${id}`)
    );

    if (!isProtectedPost && !isProtectedCategory) {
      return null;
    }

    const paymentHeader = req.headers.get('X-Payment-Token');
    
    if (!paymentHeader) {
      return new Response(
        JSON.stringify({
          error: 'payment_required',
          price: '100',
          currency: config.paymentToken,
          network: config.network,
          message: 'Premium content - payment required',
        }),
        {
          status: 402,
          headers: { 
            'Content-Type': 'application/json',
            'X-Payment-Required': 'true',
          },
        }
      );
    }

    return null;
  };
}

export function initWordPressPlugin(config: WordPressX402Config) {
  return {
    name: 'x402-payment-gateway',
    version: '1.0.0',
    hooks: {
      init: () => {
        console.log('x402 plugin initialized');
      },
      rest_api_init: (rest: any) => {
        rest.registerRoute('wp/v2', '/x402/pay', {
          methods: 'POST',
          callback: async (data: any) => {
            return { success: true };
          },
        });
      },
    },
  };
}

export function shortcode_x402(args: any, content: string) {
  return `<div class="x402-protected" data-price="${args.price || '100'}">${content}</div>`;
}
