/**
 * Bitcoin Lightning Network Adapter
 * 
 * Provides Lightning Network payment support for x402 protocol.
 * Supports LND, Core Lightning, and LNURL.
 */

export interface LightningConfig {
  /** Node type: 'lnd' | 'cln' | 'lnurl' */
  nodeType: 'lnd' | 'cln' | 'lnurl';
  /** Node URL or LNURL server */
  nodeUrl: string;
  /** MACaroon (LND) or REST/Custom token */
  authToken?: string;
  /** For LND: certificate path (optional, for TLS) */
  certPath?: string;
  /** Default memo/description for invoices */
  defaultMemo?: string;
}

export interface LightningInvoice {
  /** BOLT11 invoice string */
  paymentRequest: string;
  /** Payment hash (r_hash) */
  paymentHash: string;
  /** Amount in satoshis */
  amount: number;
  /** Invoice expiry timestamp */
  expiresAt: number;
  /** Invoice state: 'open' | 'settled' | 'expired' | 'canceled' */
  state: 'open' | 'settled' | 'expired' | 'canceled';
}

export interface CreateInvoiceParams {
  amount: number;          // Amount in satoshis
  description?: string;   // Invoice description/memo
  expirySeconds?: number;  // Expiry time (default: 3600)
}

export interface VerifyResult {
  verified: boolean;
  settledAt?: number;      // Unix timestamp when paid
  paymentHash?: string;
  amount?: number;         // Amount in satoshis
  error?: string;
}

export interface LnUrlPayRequest {
  /** LNURLp URL for wallet to fetch payment request */
  lnurlp: string;
  /** Human-readable identifier */
  identifier: string;
  /** Amount in satoshis (if fixed) */
  amountSats?: number;
  /** Description */
  description?: string;
}

/**
 * LND (Lightning Network Daemon) Adapter
 */
export class LndAdapter {
  private baseUrl: string;
  private headers: HeadersInit;

  constructor(config: LightningConfig) {
    if (!config.authToken) {
      throw new Error('LND requires authToken (macaroon)');
    }
    
    // Remove trailing slash
    this.baseUrl = config.nodeUrl.replace(/\/$/, '');
    
    this.headers = {
      'Authorization': `Bearer ${config.authToken}`,
      'Content-Type': 'application/json',
    };
    
    if (config.certPath) {
      // In Node.js, you'd load the certificate here
      // For browser/edge, you'd skip TLS verification or use Web Crypto
    }
  }

  async createInvoice(params: CreateInvoiceParams): Promise<LightningInvoice> {
    const response = await fetch(`${this.baseUrl}/v1/invoices`, {
      method: 'POST',
      headers: this.headers,
      body: JSON.stringify({
        value: params.amount,
        memo: params.description || 'x402 Payment',
        expiry: params.expirySeconds || 3600,
        private: true,  // Route hints - more private
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`LND invoice creation failed: ${error}`);
    }

    const data = await response.json();
    
    return {
      paymentRequest: data.payment_request,
      paymentHash: data.r_hash,
      amount: data.value,
      expiresAt: data.expiry_time,
      state: 'open',
    };
  }

  async verifyPayment(paymentHash: string): Promise<VerifyResult> {
    // Check invoice by payment hash
    const response = await fetch(
      `${this.baseUrl}/v1/invoice/${encodeURIComponent(paymentHash)}`,
      { headers: this.headers }
    );

    if (!response.ok) {
      // Try looking up by add_index (for settled invoices)
      return this.lookupBySettledIndex(paymentHash);
    }

    const data = await response.json();
    
    if (data.state === 'settled') {
      return {
        verified: true,
        settledAt: data.settle_date,
        paymentHash: data.r_hash,
        amount: data.value,
      };
    }

    if (data.state === 'expired') {
      return { verified: false, error: 'Invoice expired' };
    }

    if (data.state === 'canceled') {
      return { verified: false, error: 'Invoice canceled' };
    }

    return { verified: false, error: 'Invoice not settled' };
  }

  private async lookupBySettledIndex(paymentHash: string): Promise<VerifyResult> {
    // LND may have settled the invoice - check recent invoices
    const response = await fetch(
      `${this.baseUrl}/v1/invoices?limit=50`,
      { headers: this.headers }
    );

    if (!response.ok) {
      return { verified: false, error: 'Invoice not found' };
    }

    const data = await response.json();
    const invoice = data.invoices?.find(
      (inv: { r_hash: string }) => inv.r_hash === paymentHash
    );

    if (invoice && invoice.state === 'settled') {
      return {
        verified: true,
        settledAt: invoice.settle_date,
        paymentHash: invoice.r_hash,
        amount: invoice.value,
      };
    }

    return { verified: false, error: 'Invoice not found or not settled' };
  }

  async getNodeInfo() {
    const response = await fetch(`${this.baseUrl}/v1/getinfo`, {
      headers: this.headers,
    });

    if (!response.ok) {
      throw new Error('Failed to get node info');
    }

    return response.json();
  }
}

/**
 * Core Lightning (c-lightning/CLN) Adapter
 */
export class ClnAdapter {
  private baseUrl: string;
  private headers: HeadersInit;

  constructor(config: LightningConfig) {
    this.baseUrl = config.nodeUrl.replace(/\/$/, '');
    this.headers = {
      'Authorization': `Bearer ${config.authToken || ''}`,
      'Content-Type': 'application/json',
    };
  }

  async createInvoice(params: CreateInvoiceParams): Promise<LightningInvoice> {
    const response = await fetch(`${this.baseUrl}/invoice`, {
      method: 'POST',
      headers: this.headers,
      body: JSON.stringify({
        amount_msat: params.amount * 1000,  // CLN uses millisatoshis
        label: `x402-${Date.now()}`,
        description: params.description || 'x402 Payment',
        expiry: params.expirySeconds || 3600,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`CLN invoice creation failed: ${error}`);
    }

    const data = await response.json();
    
    return {
      paymentRequest: data.bolt12 || data.payment_request,
      paymentHash: data.payment_hash,
      amount: params.amount,
      expiresAt: Math.floor(Date.now() / 1000) + (params.expirySeconds || 3600),
      state: 'open',
    };
  }

  async verifyPayment(paymentHash: string): Promise<VerifyResult> {
    const response = await fetch(
      `${this.baseUrl}/listinvoices/${encodeURIComponent(paymentHash)}`,
      { headers: this.headers }
    );

    if (!response.ok) {
      return { verified: false, error: 'Invoice not found' };
    }

    const data = await response.json();
    const invoice = data.invoices?.[0];

    if (!invoice) {
      return { verified: false, error: 'Invoice not found' };
    }

    if (invoice.status === 'paid') {
      return {
        verified: true,
        settledAt: invoice.paid_at,
        paymentHash: invoice.payment_hash,
        amount: invoice.amount_msat / 1000,
      };
    }

    if (invoice.status === 'expired') {
      return { verified: false, error: 'Invoice expired' };
    }

    if (invoice.status === 'canceled') {
      return { verified: false, error: 'Invoice canceled' };
    }

    return { verified: false, error: 'Invoice not paid' };
  }
}

/**
 * LNURL Pay Request - for wallet-initiated payments
 */
export class LnUrlAdapter {
  private serverUrl: string;
  private adapter: LndAdapter | ClnAdapter;
  private config: LightningConfig;

  constructor(config: LightningConfig) {
    this.config = config;
    this.serverUrl = config.nodeUrl;
    
    if (config.nodeType === 'lnd') {
      this.adapter = new LndAdapter(config);
    } else {
      this.adapter = new ClnAdapter(config);
    }
  }

  /**
   * Generate LNURLp (LNURL-pay) URL for the given amount
   */
  async createLnUrlPay(amountSats: number, description?: string): Promise<string> {
    // Encode the callback URL with the payment parameters
    // In production, this would be your LNURL server endpoint
    const callbackParams = new URLSearchParams({
      amount: (amountSats * 1000).toString(),  // millisats
      comment: description || 'x402 Payment',
    });
    
    // For a full implementation, you'd host a LNURLp server
    // This returns a standardized LNURL encoding
    const lnurl = Buffer.from(
      `${this.serverUrl}/lnurlp/callback?${callbackParams}`
    ).toString('base64url');
    
    return `lightning:${lnurl}`;
  }

  /**
   * Create an invoice and return LNURL-encoded payment request
   */
  async createLnUrlInvoice(params: CreateInvoiceParams): Promise<LnUrlPayRequest> {
    const invoice = await this.adapter.createInvoice(params);
    
    return {
      lnurlp: await this.createLnUrlPay(params.amount, params.description),
      identifier: this.config.nodeUrl,
      amountSats: params.amount,
      description: params.description,
    };
  }
}

/**
 * Unified Lightning Adapter - automatically selects backend
 */
export class LightningAdapter {
  private lnd?: LndAdapter;
  private cln?: ClnAdapter;
  private lnurl?: LnUrlAdapter;
  private config: LightningConfig;

  constructor(config: LightningConfig) {
    this.config = config;
    
    switch (config.nodeType) {
      case 'lnd':
        this.lnd = new LndAdapter(config);
        break;
      case 'cln':
        this.cln = new ClnAdapter(config);
        break;
      case 'lnurl':
        this.lnurl = new LnUrlAdapter(config);
        break;
    }
  }

  async createInvoice(params: CreateInvoiceParams): Promise<LightningInvoice> {
    if (this.lnd) {
      return this.lnd.createInvoice(params);
    }
    if (this.cln) {
      return this.cln.createInvoice(params);
    }
    throw new Error('No Lightning backend configured');
  }

  async verifyPayment(paymentHash: string): Promise<VerifyResult> {
    if (this.lnd) {
      return this.lnd.verifyPayment(paymentHash);
    }
    if (this.cln) {
      return this.cln.verifyPayment(paymentHash);
    }
    throw new Error('No Lightning backend configured');
  }

  async getNodeInfo() {
    if (this.lnd) {
      return this.lnd.getNodeInfo();
    }
    throw new Error('getNodeInfo not supported for this backend');
  }

  /**
   * Get LNURL for payment
   */
  async createLnUrl(amountSats: number, description?: string): Promise<string> {
    if (this.lnurl) {
      return this.lnurl.createLnUrlPay(amountSats, description);
    }
    throw new Error('LNURL not configured');
  }
}

/**
 * Factory function to create Lightning adapter
 */
export function createLightningAdapter(config: LightningConfig): LightningAdapter {
  return new LightningAdapter(config);
}

/**
 * Convert fiat to satoshis (requires price feed)
 */
export function fiatToSatoshis(fiatAmount: number, btcPrice: number): number {
  // btcPrice in USD
  const btcInSatoshis = 100000000;  // 1 BTC = 100M sats
  const satsPerDollar = btcInSatoshis / btcPrice;
  return Math.floor(fiatAmount * satsPerDollar);
}

/**
 * Convert USD to satoshis
 */
export function usdToSatoshis(usdAmount: number, btcPrice: number = 67000): number {
  return fiatToSatoshis(usdAmount, btcPrice);
}

/**
 * Create x402 payment requirement for Lightning
 */
export interface LightningPaymentRequirement {
  protocol: 'x402';
  scheme: 'v1-lightning';
  network: 'lightning';
  paymentToken: 'BTC';
  amount: number;        // in satoshis
  recipient: string;    // node public key or LNURL
  paymentRequest: string; // BOLT11 invoice
  expiresAt: number;
}

export function createLightningPaymentRequired(
  amountSats: number,
  recipient: string,
  invoice: LightningInvoice,
  description?: string
): LightningPaymentRequirement {
  return {
    protocol: 'x402',
    scheme: 'v1-lightning',
    network: 'lightning',
    paymentToken: 'BTC',
    amount: amountSats,
    recipient: recipient,
    paymentRequest: invoice.paymentRequest,
    expiresAt: invoice.expiresAt,
  };
}

/**
 * Middleware for Lightning payments in x402 flow
 */
export interface LightningMiddlewareConfig {
  adapter: LightningAdapter;
  recipient: string;  // Your node's public key
  defaultMemo?: string;
  pricePerRequest: number;  // In sats
  btcPrice?: number;        // Current BTC price for USD conversion
}

export function createLightningMiddleware(config: LightningMiddlewareConfig) {
  return async (req: Request): Promise<Response | null> => {
    const paymentHeader = req.headers.get('X-Payment-Token');
    
    if (!paymentHeader) {
      // Create invoice for the required amount
      const amountSats = config.btcPrice 
        ? usdToSatoshis(config.pricePerRequest, config.btcPrice)
        : config.pricePerRequest;
      
      const invoice = await config.adapter.createInvoice({
        amount: amountSats,
        description: config.defaultMemo || 'API Access',
        expirySeconds: 3600,
      });
      
      const paymentReq = createLightningPaymentRequired(
        amountSats,
        config.recipient,
        invoice,
        config.defaultMemo
      );
      
      return new Response(
        JSON.stringify({
          error: 'payment_required',
          ...paymentReq,
        }),
        {
          status: 402,
          headers: {
            'Content-Type': 'application/json',
            'X-Payment-Required': 'true',
            'X-Amount-Sats': amountSats.toString(),
            'X-Payment-Request': invoice.paymentRequest,
          },
        }
      );
    }
    
    // Verify payment token (should be payment hash or preimage)
    const result = await config.adapter.verifyPayment(paymentHeader);
    
    if (!result.verified) {
      return new Response(
        JSON.stringify({ error: 'payment_invalid', details: result.error }),
        { status: 402 }
      );
    }
    
    return null;  // Payment verified, continue to handler
  };
}

export type {
  LightningConfig as Config,
  LightningInvoice as Invoice,
  CreateInvoiceParams as InvoiceParams,
  VerifyResult as PaymentResult,
};
