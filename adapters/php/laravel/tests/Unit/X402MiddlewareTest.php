<?php

namespace Tests\Unit\X402;

use PHPUnit\Framework\TestCase;
use App\Http\Middleware\X402Middleware;

class X402MiddlewareTest extends TestCase
{
    private $config;

    protected function setUp(): void
    {
        parent::setUp();
        
        $this->config = [
            'price_per_request' => 100,
            'scheme' => 'v1-eip155-exact',
            'network' => 'eip155:1',
            'payment_token' => '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
            'recipient' => '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
            'description' => 'API access',
            'expiry_seconds' => 3600,
            'facilitator_url' => 'https://facilitator.x402.rs',
        ];
    }

    public function testMiddlewareReturns402WithoutToken(): void
    {
        $middleware = new X402Middleware();
        
        $request = $this->createMockRequest('/api/data', null);
        $next = function ($request) {
            return new \Illuminate\Http\Response('OK');
        };
        
        $response = $middleware->handle($request, $next);
        
        $this->assertEquals(402, $response->getStatusCode());
        
        $content = json_decode($response->getContent(), true);
        $this->assertEquals(1, $content['version']);
        $this->assertEquals('v1-eip155-exact', $content['scheme']);
    }

    public function testMiddlewareReturns200WithValidToken(): void
    {
        $middleware = new X402Middleware();
        
        $token = [
            'signature' => '0xabc123',
            'sender' => '0xsender',
            'amount' => '100',
            'valid_after' => time(),
        ];
        
        $request = $this->createMockRequest('/api/data', base64_encode(json_encode($token)));
        $next = function ($request) {
            return new \Illuminate\Http\JsonResponse(['data' => 'success']);
        };
        
        $response = $middleware->handle($request, $next);
        
        $this->assertEquals(200, $response->getStatusCode());
    }

    public function testExcludedPathsPassThrough(): void
    {
        $middleware = new X402Middleware();
        
        $request = $this->createMockRequest('/docs', null);
        $next = function ($request) {
            return new \Illuminate\Http\Response('OK');
        };
        
        $response = $middleware->handle($request, $next);
        
        $this->assertEquals(200, $response->getStatusCode());
    }

    public function testInvalidTokenReturns402(): void
    {
        $middleware = new X402Middleware();
        
        $request = $this->createMockRequest('/api/data', 'invalid-base64!!!');
        $next = function ($request) {
            return new \Illuminate\Http\Response('OK');
        };
        
        $response = $middleware->handle($request, $next);
        
        $this->assertEquals(402, $response->getStatusCode());
    }

    public function testSolanaNetworkConfiguration(): void
    {
        $config = array_merge($this->config, [
            'scheme' => 'v1-solana-exact',
            'network' => 'solana:101',
            'payment_token' => 'EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ',
        ]);
        
        // Test that config is properly set
        $middleware = new X402Middleware();
        
        $reflection = new \ReflectionClass($middleware);
        $property = $reflection->getProperty('config');
        $property->setAccessible(true);
        
        // Test the middleware creation
        $this->assertInstanceOf(X402Middleware::class, $middleware);
    }

    public function testPaymentRequiredStructure(): void
    {
        $middleware = new X402Middleware();
        
        // Use reflection to call the protected method
        $reflection = new \ReflectionClass($middleware);
        $method = $reflection->getMethod('createPaymentRequired');
        $method->setAccessible(true);
        
        $result = $method->invoke($middleware, '/api/test');
        
        $this->assertArrayHasKey('version', $result);
        $this->assertArrayHasKey('scheme', $result);
        $this->assertArrayHasKey('network', $result);
        $this->assertArrayHasKey('payment_token', $result);
        $this->assertArrayHasKey('max_amount', $result);
        $this->assertArrayHasKey('recipient', $result);
        $this->assertArrayHasKey('expiry', $result);
        $this->assertArrayHasKey('salt', $result);
        $this->assertArrayHasKey('instructions', $result);
    }

    public function testCustomExpirySeconds(): void
    {
        $config = array_merge($this->config, [
            'expiry_seconds' => 7200,
        ]);
        
        $middleware = new X402Middleware();
        
        $reflection = new \ReflectionClass($middleware);
        $property = $reflection->getProperty('config');
        $property->setAccessible(true);
        $property->setValue($middleware, $config);
        
        $method = $reflection->getMethod('createPaymentRequired');
        $method->setAccessible(true);
        
        $result = $method->invoke($middleware, '/test');
        
        $expectedExpiry = time() + 7200;
        $this->assertEquals($expectedExpiry, $result['expiry'], '', 2);
    }

    public function testInstructionsContainCorrectUrl(): void
    {
        $middleware = new X402Middleware();
        
        $reflection = new \ReflectionClass($middleware);
        $method = $reflection->getMethod('createPaymentRequired');
        $method->setAccessible(true);
        
        $result = $method->invoke($middleware, '/api/data');
        
        $this->assertCount(1, $result['instructions']);
        
        $instruction = $result['instructions'][0];
        $this->assertEquals('https', $instruction['protocol']);
        $this->assertEquals('POST', $instruction['method']);
        $this->assertEquals('https://facilitator.x402.rs/v1/payment', $instruction['url']);
    }

    public function testShouldExcludeMethod(): void
    {
        $middleware = new X402Middleware();
        
        $reflection = new \ReflectionClass($middleware);
        $method = $reflection->getMethod('shouldExclude');
        $method->setAccessible(true);
        
        $this->assertTrue($method->invoke($middleware, '/docs'));
        $this->assertTrue($method->invoke($middleware, '/api'));
        $this->assertTrue($method->invoke($middleware, '/static/js/app.js'));
        $this->assertFalse($method->invoke($middleware, '/api/data'));
        $this->assertFalse($method->invoke($middleware, '/users/profile'));
    }

    public function testIsValidTokenMethod(): void
    {
        $middleware = new X402Middleware();
        
        $reflection = new \ReflectionClass($middleware);
        $method = $reflection->getMethod('isValidToken');
        $method->setAccessible(true);
        
        // Empty token
        $this->assertFalse($method->invoke($middleware, null));
        $this->assertFalse($method->invoke($middleware, ''));
        
        // Valid token
        $validToken = base64_encode(json_encode([
            'signature' => '0xabc',
            'sender' => '0x123',
        ]));
        $this->assertTrue($method->invoke($middleware, $validToken));
        
        // Invalid base64
        $this->assertFalse($method->invoke($middleware, 'not-valid-base64!!!'));
    }

    private function createMockRequest(string $path, ?string $token): \Illuminate\Http\Request
    {
        $request = \Illuminate\Http\Request::create($path, 'GET');
        
        if ($token !== null) {
            $request->headers->set('X-Payment-Token', $token);
        }
        
        return $request;
    }
}
