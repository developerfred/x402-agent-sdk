/**
 * Tempo.xyz adapter for x402
 * 
 * Tempo is Stripe + Paradigm's blockchain for payments with:
 * - Native Stripe integration (accept fiat + crypto)
 * - Machine Payments Protocol (MPP) for AI agents
 * - Sessions: OAuth for money - pre-authorize spending caps
 */

export enum TempoNetwork {
  Mainnet = 5000,
  Testnet = 5001,
  Devnet = 5002,
}

export enum TempoToken {
  USDC = 'USDC',
  USDT = 'USDT',
  EUROC = 'EUROC',
  GBPC = 'GBPC',
}

export interface TempoPaymentOptions {
  network: TempoNetwork;
  recipient: string;
  token: TempoToken;
  amount: bigint;
  description?: string;
}

export interface TempoSession {
  sessionId: string;
  authorizedSpender: string;
  maxAmount: bigint;
  spent: bigint;
  expiresAt: number;
  token: TempoToken;
}

export interface TempoStripeConfig {
  stripePublishableKey: string;
  merchantId: string;
  enableFiat: boolean;
  enableCrypto: boolean;
}

export class TempoPayment {
  readonly network: TempoNetwork;
  readonly recipient: string;
  readonly token: TempoToken;
  readonly amount: bigint;
  readonly description?: string;

  constructor(options: TempoPaymentOptions) {
    this.network = options.network;
    this.recipient = options.recipient;
    this.token = options.token;
    this.amount = options.amount;
    this.description = options.description;
  }

  /**
   * Convert to x402 PaymentRequired format
   */
  toPaymentRequired(): object {
    const networkId = `tempo:${this.network}`;
    
    return {
      version: 1,
      scheme: 'v1-tempo-exact',
      network: networkId,
      payment_token: this.token,
      max_amount: this.amount.toString(),
      recipient: this.recipient,
      description: this.description,
      expires_at: undefined,
    };
  }

  /**
   * Get the HTTP 402 response headers
   */
  toHeaders(): Record<string, string> {
    return {
      'X-Payment-Required': 'true',
      'X-Price': this.amount.toString(),
      'X-Currency': this.token,
      'X-Network': `tempo:${this.network}`,
    };
  }
}

export class TempoSessionManager {
  private sessions: Map<string, TempoSession> = new Map();

  /**
   * Create a new session
   */
  createSession(
    authorizedSpender: string,
    maxAmount: bigint,
    token: TempoToken,
    expiresInSeconds: number = 3600
  ): TempoSession {
    const sessionId = `sess_${Date.now()}_${Math.random().toString(36).slice(2)}`;
    const expiresAt = Math.floor(Date.now() / 1000) + expiresInSeconds;

    const session: TempoSession = {
      sessionId,
      authorizedSpender,
      maxAmount,
      spent: 0n,
      expiresAt,
      token,
    };

    this.sessions.set(sessionId, session);
    return session;
  }

  /**
   * Get session by ID
   */
  getSession(sessionId: string): TempoSession | undefined {
    return this.sessions.get(sessionId);
  }

  /**
   * Check if session is valid
   */
  isValid(sessionId: string): boolean {
    const session = this.sessions.get(sessionId);
    if (!session) return false;

    const now = Math.floor(Date.now() / 1000);
    return now < session.expiresAt && session.spent < session.maxAmount;
  }

  /**
   * Check if amount can be spent
   */
  canSpend(sessionId: string, amount: bigint): boolean {
    const session = this.sessions.get(sessionId);
    if (!session) return false;

    return this.isValid(sessionId) && (session.spent + amount) <= session.maxAmount;
  }

  /**
   * Record a payment
   */
  recordPayment(sessionId: string, amount: bigint): boolean {
    const session = this.sessions.get(sessionId);
    if (!session || !this.canSpend(sessionId, amount)) {
      return false;
    }

    session.spent += amount;
    return true;
  }

  /**
   * Get remaining amount
   */
  remaining(sessionId: string): bigint {
    const session = this.sessions.get(sessionId);
    if (!session) return 0n;

    return session.maxAmount - session.spent;
  }

  /**
   * Revoke a session
   */
  revoke(sessionId: string): boolean {
    return this.sessions.delete(sessionId);
  }
}

/**
 * Create Tempo payment for x402 response
 */
export function createTempoPayment(options: TempoPaymentOptions): TempoPayment {
  return new TempoPayment(options);
}

/**
 * Check if request supports MPP (Machine Payments Protocol)
 */
export function supportsMPP(headers: Headers): boolean {
  const wwwAuth = headers.get('WWW-Authenticate');
  return wwwAuth?.includes('Payment') ?? false;
}

/**
 * Get supported protocols from response
 */
export function getSupportedProtocols(headers: Headers): string[] {
  const wwwAuth = headers.get('WWW-Authenticate');
  if (!wwwAuth) return [];

  const protocols: string[] = [];
  
  if (wwwAuth.includes('x402')) {
    protocols.push('x402');
  }
  if (wwwAuth.includes('mpp')) {
    protocols.push('mpp');
  }

  return protocols;
}
