<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class X402Middleware
{
    protected $config;

    public function __construct()
    {
        $this->config = [
            'price_per_request' => config('x402.price_per_request', 100),
            'scheme' => config('x402.scheme', 'v1-eip155-exact'),
            'network' => config('x402.network', 'eip155:1'),
            'payment_token' => config('x402.payment_token', ''),
            'recipient' => config('x402.recipient', ''),
            'description' => config('x402.description', 'API access'),
            'expiry_seconds' => config('x402.expiry_seconds', 3600),
            'facilitator_url' => config('x402.facilitator_url', 'https://facilitator.x402.rs'),
        ];
    }

    public function handle(Request $request, Closure $next): Response
    {
        $path = $request->path();

        if ($this->shouldExclude($path)) {
            return $next($request);
        }

        $tokenHeader = $request->header('X-Payment-Token');

        if ($this->isValidToken($tokenHeader)) {
            return $next($request);
        }

        $paymentRequired = $this->createPaymentRequired($path);

        return response()->json($paymentRequired, 402);
    }

    protected function shouldExclude(string $path): bool
    {
        $excludePaths = ['docs', 'api', 'static', 'storage'];
        
        foreach ($excludePaths as $excludePath) {
            if (str_starts_with($path, $excludePath)) {
                return true;
            }
        }
        
        return false;
    }

    protected function isValidToken(?string $token): bool
    {
        if (empty($token)) {
            return false;
        }

        try {
            $decoded = base64_decode($token);
            $data = json_decode($decoded, true);
            
            return isset($data['signature']) && isset($data['sender']);
        } catch (\Exception $e) {
            return false;
        }
    }

    protected function createPaymentRequired(string $path): array
    {
        $now = time();
        $expiry = $now + $this->config['expiry_seconds'];
        $salt = substr(md5("{$path}:{$now}"), 0, 16);

        return [
            'version' => 1,
            'scheme' => $this->config['scheme'],
            'network' => $this->config['network'],
            'payment_token' => $this->config['payment_token'],
            'max_amount' => (string) $this->config['price_per_request'],
            'recipient' => $this->config['recipient'],
            'description' => $this->config['description'],
            'expiry' => $expiry,
            'salt' => $salt,
            'instructions' => [
                [
                    'protocol' => 'https',
                    'method' => 'POST',
                    'url' => $this->config['facilitator_url'] . '/v1/payment',
                    'headers' => [
                        'Content-Type' => 'application/json',
                    ],
                    'body' => [
                        'network' => $this->config['network'],
                        'payment_token' => $this->config['payment_token'],
                        'amount' => (string) $this->config['price_per_request'],
                        'recipient' => $this->config['recipient'],
                        'valid_after' => $now,
                        'salt' => $salt,
                    ],
                ],
            ],
        ];
    }
}
