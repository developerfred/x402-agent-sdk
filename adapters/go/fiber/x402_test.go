package fiber

import (
	"encoding/base64"
	"encoding/json"
	"net/http/httptest"
	"testing"
	"time"

	fiber "github.com/gofiber/fiber/v2"
)

func TestFiberConfig(t *testing.T) {
	cfg := Config{
		PricePerRequest: 100,
		Network:         "eip155:1",
		PaymentToken:    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
		Recipient:       "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
	}

	if cfg.PricePerRequest != 100 {
		t.Errorf("expected 100, got %d", cfg.PricePerRequest)
	}
}

func TestFiberDefaultConfig(t *testing.T) {
	cfg := DefaultConfig()

	if cfg.PricePerRequest != 100 {
		t.Errorf("expected 100, got %d", cfg.PricePerRequest)
	}
	if cfg.Scheme != "v1-eip155-exact" {
		t.Errorf("expected scheme, got %s", cfg.Scheme)
	}
}

func TestFiberCreatePaymentRequired(t *testing.T) {
	cfg := Config{
		PricePerRequest: 100,
		Network:         "eip155:1",
		PaymentToken:    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
		Recipient:       "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
	}

	pr := CreatePaymentRequired(cfg, "/api/test")

	if pr.Version != 1 {
		t.Errorf("expected version 1, got %d", pr.Version)
	}
	if pr.Scheme != "v1-eip155-exact" {
		t.Errorf("expected scheme, got %s", pr.Scheme)
	}
	if pr.MaxAmount != "100" {
		t.Errorf("expected amount 100, got %s", pr.MaxAmount)
	}
}

func TestFiberCreateSolanaPayment(t *testing.T) {
	cfg := Config{
		PricePerRequest: 50,
		Scheme:          "v1-solana-exact",
		Network:         "solana:101",
		PaymentToken:    "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
		Recipient:       "SolanaAddr",
	}

	pr := CreatePaymentRequired(cfg, "/api/solana")

	if pr.Scheme != "v1-solana-exact" {
		t.Errorf("expected solana scheme, got %s", pr.Scheme)
	}
	if pr.Network != "solana:101" {
		t.Errorf("expected solana network, got %s", pr.Network)
	}
}

func TestFiberParsePaymentToken(t *testing.T) {
	app := fiber.New()

	token := PaymentToken{
		Signature:  "0xsig",
		Sender:     "0xsender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}
	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	app.Get("/test", func(c *fiber.Ctx) error {
		result := ParsePaymentToken(c)
		if result == nil {
			return c.Status(400).SendString("nil")
		}
		return c.JSON(result)
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("X-Payment-Token", encoded)

	resp, err := app.Test(req)
	if err != nil {
		t.Fatalf("failed to test: %v", err)
	}

	if resp.StatusCode != 200 {
		t.Errorf("expected 200, got %d", resp.StatusCode)
	}
}

func TestFiberPaymentTokenValid(t *testing.T) {
	valid := &PaymentToken{
		Signature:  "sig",
		Sender:     "sender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}

	if !valid.IsValid() {
		t.Error("expected valid token")
	}

	invalid := &PaymentToken{}
	if invalid.IsValid() {
		t.Error("expected invalid token")
	}
}

func TestFiberMiddlewareNoPayment(t *testing.T) {
	app := fiber.New()

	cfg := DefaultConfig()
	cfg.Network = "eip155:1"
	cfg.PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
	cfg.Recipient = "0x742d"

	app.Use(Middleware(cfg))
	app.Get("/api/data", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"data": "success"})
	})

	req := httptest.NewRequest("GET", "/api/data", nil)
	resp, err := app.Test(req)
	if err != nil {
		t.Fatalf("failed: %v", err)
	}

	if resp.StatusCode != fiber.StatusPaymentRequired {
		t.Errorf("expected 402, got %d", resp.StatusCode)
	}
}

func TestFiberMiddlewareWithToken(t *testing.T) {
	app := fiber.New()

	cfg := DefaultConfig()
	cfg.Network = "eip155:1"
	cfg.PaymentToken = "0xABC"
	cfg.Recipient = "0x123"

	app.Use(Middleware(cfg))
	app.Get("/api/data", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"data": "success"})
	})

	token := PaymentToken{
		Signature:  "valid",
		Sender:     "sender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}
	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	req := httptest.NewRequest("GET", "/api/data", nil)
	req.Header.Set("X-Payment-Token", encoded)

	resp, _ := app.Test(req)

	if resp.StatusCode != fiber.StatusOK {
		t.Errorf("expected 200, got %d", resp.StatusCode)
	}
}

func TestFiberRequirePayment(t *testing.T) {
	app := fiber.New()

	cfg := DefaultConfig()

	app.Get("/premium", RequirePayment(cfg), func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"premium": "content"})
	})

	// No token
	req := httptest.NewRequest("GET", "/premium", nil)
	resp, _ := app.Test(req)
	if resp.StatusCode != fiber.StatusPaymentRequired {
		t.Errorf("expected 402, got %d", resp.StatusCode)
	}

	// With token
	token := PaymentToken{
		Signature:  "sig",
		Sender:     "sender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}
	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	req2 := httptest.NewRequest("GET", "/premium", nil)
	req2.Header.Set("X-Payment-Token", encoded)
	resp2, _ := app.Test(req2)

	if resp2.StatusCode != fiber.StatusOK {
		t.Errorf("expected 200, got %d", resp2.StatusCode)
	}
}
