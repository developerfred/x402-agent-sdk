<?php

namespace App\X402;

use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\HttpKernel\Event\RequestEvent;
use Symfony\Component\HttpKernel\KernelEvents;

class X402Middleware
{
    private $config;

    public function __construct(array $config = [])
    {
        $this->config = array_merge([
            'price_per_request' => 100,
            'scheme' => 'v1-eip155-exact',
            'network' => 'eip155:1',
            'payment_token' => '',
            'recipient' => '',
            'description' => 'API access',
            'expiry_seconds' => 3600,
            'facilitator_url' => 'https://facilitator.x402.rs',
        ], $config);
    }

    public static function getSubscribedEvents(): array
    {
        return [
            KernelEvents::REQUEST => ['onKernelRequest', 10],
        ];
    }

    public function onKernelRequest(RequestEvent $event): void
    {
        $request = $event->getRequest();
        $path = $request->getPathInfo();

        if ($this->shouldExclude($path)) {
            return;
        }

        $tokenHeader = $request->headers->get('X-Payment-Token');

        if ($this->isValidToken($tokenHeader)) {
            return;
        }

        $paymentRequired = $this->createPaymentRequired($path);

        $response = new Response(
            json_encode($paymentRequired),
            402,
            ['Content-Type' => 'application/json']
        );

        $event->setResponse($response);
    }

    private function shouldExclude(string $path): bool
    {
        $excludePaths = ['/docs', '/api', '/static', '/_wdt', '_profiler'];

        foreach ($excludePaths as $excludePath) {
            if (str_starts_with($path, $excludePath)) {
                return true;
            }
        }

        return false;
    }

    private function isValidToken(?string $token): bool
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

    private function createPaymentRequired(string $path): array
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
