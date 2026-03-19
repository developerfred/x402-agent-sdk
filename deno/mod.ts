export * from "./types.ts";

export interface X402ClientOptions {
  walletPrivateKey?: string;
  facilitatorUrl?: string;
  maxRetries?: number;
  timeoutSeconds?: number;
}

export class X402Client {
  private walletPrivateKey?: string;
  private facilitatorUrl: string;
  private maxRetries: number;
  private timeoutSeconds: number;

  constructor(options: X402ClientOptions = {}) {
    this.walletPrivateKey = options.walletPrivateKey;
    this.facilitatorUrl = options.facilitatorUrl || "https://facilitator.x402.rs";
    this.maxRetries = options.maxRetries || 3;
    this.timeoutSeconds = options.timeoutSeconds || 30;
  }

  async request(
    url: string,
    options: RequestInit & { paymentRequired?: PaymentRequired } = {},
  ): Promise<Response> {
    const { paymentRequired, ...fetchOptions } = options;

    const response = await fetch(url, {
      ...fetchOptions,
      headers: {
        ...fetchOptions.headers,
        "X-402-Version": "1",
      },
    });

    if (response.status === 402) {
      const headerValue = response.headers.get("X-Payment-Required");
      if (!headerValue) {
        throw new Error("Payment required but no payment header found");
      }

      const pr = JSON.parse(headerValue) as PaymentRequired;
      const token = await this.createPaymentToken(pr);
      
      return fetch(url, {
        ...fetchOptions,
        headers: {
          ...fetchOptions.headers,
          "X-Payment-Token": JSON.stringify(token),
          "X-402-Version": "1",
        },
      });
    }

    return response;
  }

  private async createPaymentToken(paymentRequired: PaymentRequired): Promise<PaymentToken> {
    return {
      id: crypto.randomUUID(),
      payment_required: paymentRequired,
      signature: "placeholder_signature",
      sender: this.walletPrivateKey ? this.deriveAddress(this.walletPrivateKey) : "0xsender",
      amount: paymentRequired.max_amount,
      nonce: Date.now(),
      created_at: new Date().toISOString(),
    };
  }

  private deriveAddress(privateKey: string): string {
    return `0x${privateKey.slice(0, 40)}`;
  }
}

export class X402Server {
  private pricePerRequest: number;
  private scheme: string;
  private network: string;
  private paymentToken: string;
  private recipient: string;

  constructor(
    pricePerRequest: number,
    scheme: string,
    network: string,
    paymentToken: string,
    recipient: string,
  ) {
    this.pricePerRequest = pricePerRequest;
    this.scheme = scheme;
    this.network = network;
    this.paymentToken = paymentToken;
    this.recipient = recipient;
  }

  createPaymentRequired(description?: string): PaymentRequired {
    return {
      version: 1,
      scheme: this.scheme,
      network: this.network,
      payment_token: this.paymentToken,
      max_amount: this.pricePerRequest.toString(),
      recipient: this.recipient,
      description,
    };
  }

  checkPayment(headers: Headers): boolean {
    const paymentRequired = headers.get("X-Payment-Required");
    const paymentToken = headers.get("X-Payment-Token");

    if (!paymentRequired && !paymentToken) {
      return false;
    }

    if (paymentToken) {
      try {
        const token = JSON.parse(paymentToken) as PaymentToken;
        return token.amount === this.pricePerRequest.toString();
      } catch {
        return false;
      }
    }

    return false;
  }

  middleware() {
    return async (request: Request): Promise<Response> => {
      if (!this.checkPayment(request.headers)) {
        const pr = this.createPaymentRequired();
        return new Response(JSON.stringify(pr), {
          status: 402,
          headers: {
            "X-Payment-Required": JSON.stringify(pr),
            "Content-Type": "application/json",
          },
        });
      }

      return new Response("OK", { status: 200 });
    };
  }
}

export function createSolanaPayment(
  network: "mainnet" | "devnet" | "testnet",
  recipient: string,
  token: "SOL" | "USDC" | "USDT",
  amount: number,
): PaymentRequired {
  const chainId = network === "mainnet" ? 101 : network === "devnet" ? 102 : 103;
  
  return {
    version: 1,
    scheme: "v1-solana-exact",
    network: `solana:${chainId}`,
    payment_token: token,
    max_amount: amount.toString(),
    recipient,
  };
}

export function createStacksPayment(
  network: "mainnet" | "testnet" | "devnet",
  recipient: string,
  token: "STX" | "SIP010",
  amount: number,
): PaymentRequired {
  const chainId = network === "mainnet" ? "0x01" : network === "testnet" ? "0x80000000" : "devnet";
  
  return {
    version: 1,
    scheme: "v2-stacks-exact",
    network: `stacks:${chainId}`,
    payment_token: token,
    max_amount: amount.toString(),
    recipient,
  };
}
