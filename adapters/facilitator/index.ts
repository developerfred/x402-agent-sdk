/**
 * Payment Facilitator - Real payment verification
 * 
 * Connects to payment providers: Stripe, Coinbase, Tempo
 */

export interface PaymentProvider {
  name: string;
  verifyPayment(payment: Payment): Promise<PaymentResult>;
  createPaymentLink(payment: CreatePaymentParams): Promise<PaymentLink>;
}

export interface Payment {
  txHash: string;
  amount: string;
  token: string;
  network: string;
  recipient: string;
  from: string;
}

export interface PaymentResult {
  verified: boolean;
  blockNumber?: number;
  timestamp?: number;
  error?: string;
}

export interface CreatePaymentParams {
  amount: string;
  token: string;
  recipient: string;
  description?: string;
  expiresIn?: number;
}

export interface PaymentLink {
  url: string;
  id: string;
  expiresAt: number;
}

export class StripeFacilitator implements PaymentProvider {
  name = 'stripe';
  private secretKey: string;
  private baseUrl = 'https://api.stripe.com/v1';

  constructor(secretKey: string) {
    this.secretKey = secretKey;
  }

  async verifyPayment(payment: Payment): Promise<PaymentResult> {
    try {
      const response = await fetch(`${this.baseUrl}/payment_intents/${payment.txHash}`, {
        headers: {
          'Authorization': `Bearer ${this.secretKey}`,
        },
      });

      if (!response.ok) {
        return { verified: false, error: 'Payment not found' };
      }

      const data = await response.json();
      
      return {
        verified: data.status === 'succeeded',
        timestamp: Math.floor(Date.now() / 1000),
      };
    } catch (error) {
      return { verified: false, error: String(error) };
    }
  }

  async createPaymentLink(params: CreatePaymentParams): Promise<PaymentLink> {
    const response = await fetch(`${this.baseUrl}/payment_links`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.secretKey}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        'line_items[0][price_data][currency]': 'usd',
        'line_items[0][price_data][unit_amount]': params.amount,
        'line_items[0][price_data][product_data][name]': params.description || 'API Access',
        'line_items[0][quantity]': '1',
      }),
    });

    const data = await response.json();
    
    return {
      url: data.url,
      id: data.id,
      expiresAt: Date.now() + (params.expiresIn || 3600) * 1000,
    };
  }
}

export class CoinbaseFacilitator implements PaymentProvider {
  name = 'coinbase';
  private apiKey: string;
  private baseUrl = 'https://api.commerce.coinbase.com';

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async verifyPayment(payment: Payment): Promise<PaymentResult> {
    try {
      const response = await fetch(`${this.baseUrl}/charges/${payment.txHash}`, {
        headers: {
          'X-CC-Api-Key': this.apiKey,
        },
      });

      if (!response.ok) {
        return { verified: false, error: 'Payment not found' };
      }

      const data = await response.json();
      
      return {
        verified: data.data.status === 'confirmed',
        timestamp: Math.floor(new Date(data.data.timeline[0].time).getTime() / 1000),
      };
    } catch (error) {
      return { verified: false, error: String(error) };
    }
  }

  async createPaymentLink(params: CreatePaymentParams): Promise<PaymentLink> {
    const response = await fetch(`${this.baseUrl}/charges`, {
      method: 'POST',
      headers: {
        'X-CC-Api-Key': this.apiKey,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: params.description || 'API Access',
        description: 'Payment for API access',
        pricing_type: 'fixed_price',
        local_price: {
          amount: params.amount,
          currency: 'USD',
        },
      }),
    });

    const data = await response.json();
    
    return {
      url: data.data.hosted_url,
      id: data.data.code,
      expiresAt: Date.now() + (params.expiresIn || 3600) * 1000,
    };
  }
}

export class TempoFacilitator implements PaymentProvider {
  name = 'tempo';
  private apiKey: string;
  private baseUrl = 'https://api.tempo.xyz/v1';

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async verifyPayment(payment: Payment): Promise<PaymentResult> {
    try {
      const response = await fetch(`${this.baseUrl}/payments/verify`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          transaction_hash: payment.txHash,
          network: payment.network,
        }),
      });

      if (!response.ok) {
        return { verified: false, error: 'Payment verification failed' };
      }

      const data = await response.json();
      
      return {
        verified: data.verified,
        blockNumber: data.block_number,
        timestamp: data.timestamp,
      };
    } catch (error) {
      return { verified: false, error: String(error) };
    }
  }

  async createPaymentLink(params: CreatePaymentParams): Promise<PaymentLink> {
    const response = await fetch(`${this.baseUrl}/payments/sessions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        amount: params.amount,
        token: params.token,
        recipient: params.recipient,
        description: params.description,
        expires_in: params.expiresIn || 3600,
      }),
    });

    const data = await response.json();
    
    return {
      url: data.session_url,
      id: data.session_id,
      expiresAt: Date.now() + (params.expiresIn || 3600) * 1000,
    };
  }
}

export class MultiProviderFacade implements PaymentProvider {
  name = 'multi';
  private providers: Map<string, PaymentProvider> = new Map();

  addProvider(network: string, provider: PaymentProvider) {
    this.providers.set(network, provider);
  }

  async verifyPayment(payment: Payment): Promise<PaymentResult> {
    const provider = this.providers.get(payment.network);
    
    if (!provider) {
      return { verified: false, error: `No provider for network: ${payment.network}` };
    }

    return provider.verifyPayment(payment);
  }

  async createPaymentLink(params: CreatePaymentParams): Promise<PaymentLink> {
    const provider = this.providers.get(params.token);
    
    if (!provider) {
      throw new Error(`No provider for token: ${params.token}`);
    }

    return provider.createPaymentLink(params);
  }
}

export function createFacilitator(type: 'stripe' | 'coinbase' | 'tempo', apiKey: string): PaymentProvider {
  switch (type) {
    case 'stripe':
      return new StripeFacilitator(apiKey);
    case 'coinbase':
      return new CoinbaseFacilitator(apiKey);
    case 'tempo':
      return new TempoFacilitator(apiKey);
    default:
      throw new Error(`Unknown facilitator type: ${type}`);
  }
}
