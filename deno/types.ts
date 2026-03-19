export interface PaymentRequired {
  version: number;
  scheme: string;
  network: string;
  payment_token: string;
  max_amount: string;
  recipient: string;
  expires_at?: string;
  description?: string;
}

export interface PaymentToken {
  id: string;
  payment_required: PaymentRequired;
  signature: string;
  sender: string;
  amount: string;
  nonce: number;
  created_at: string;
}

export interface Credential {
  id: string;
  type: string[];
  issuer: string;
  subject: string;
  issued_at: string;
  expires_at?: string;
  claims: Record<string, unknown>;
}

export interface CredentialBuilder {
  issuer: string;
  subject: string;
  claims: Record<string, unknown>;
  credential_type?: string;
  expires_at?: string;
}

export type SolanaNetwork = "mainnet" | "devnet" | "testnet";
export type SolanaToken = "SOL" | "USDC" | "USDT";

export interface SolanaPayment {
  network: SolanaNetwork;
  recipient: string;
  token: SolanaToken;
  amount: number;
  signature?: string;
  transaction?: string;
}

export type StacksNetwork = "mainnet" | "testnet" | "devnet";
export type StacksToken = "STX" | "SIP010";

export interface StacksPayment {
  network: StacksNetwork;
  recipient: string;
  token: StacksToken;
  amount: number;
  signature?: string;
  transaction_id?: string;
}

export interface AgentClientConfig {
  wallet_private_key?: string;
  facilitator_url: string;
  max_retries: number;
  timeout_seconds: number;
}

export interface AgentServerConfig {
  price_per_request: number;
  scheme: string;
  network: string;
  payment_token: string;
  recipient: string;
}

export const HEADER_PAYMENT_REQUIRED = "X-Payment-Required";
export const HEADER_PAYMENT_TOKEN = "X-Payment-Token";
export const HEADER_X_402_VERSION = "X-402-Version";

export const STATUS_PAYMENT_REQUIRED = 402;

export function isPaymentRequired(status: number): boolean {
  return status === STATUS_PAYMENT_REQUIRED;
}

export function parsePaymentRequired(headerValue: string): PaymentRequired {
  return JSON.parse(headerValue);
}

export function createPaymentRequired(
  scheme: string,
  network: string,
  paymentToken: string,
  maxAmount: string,
  recipient: string,
): PaymentRequired {
  return {
    version: 1,
    scheme,
    network,
    payment_token: paymentToken,
    max_amount: maxAmount,
    recipient,
  };
}

export function encodePaymentToken(token: PaymentToken): string {
  const json = JSON.stringify(token);
  return btoa(json);
}

export function decodePaymentToken(encoded: string): PaymentToken {
  const json = atob(encoded);
  return JSON.parse(json);
}

export const SUPPORTED_NETWORKS = {
  solana: {
    mainnet: "solana:101",
    devnet: "solana:102",
    testnet: "solana:103",
  },
  stacks: {
    mainnet: "stacks:0x01",
    testnet: "stacks:0x80000000",
    devnet: "stacks:devnet",
  },
} as const;

export const SOLANA_TOKEN_ADDRESSES = {
  SOL: null,
  USDC: "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
  USDT: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11Mcx8eZ1kRBJ3",
} as const;
