package gin

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
)

func TestConfig(t *testing.T) {
	cfg := Config{
		PricePerRequest: 100,
		Network:         "eip155:1",
		PaymentToken:    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
		Recipient:       "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
	}

	if cfg.PricePerRequest != 100 {
		t.Errorf("expected 100, got %d", cfg.PricePerRequest)
	}
	if cfg.Network != "eip155:1" {
		t.Errorf("expected eip155:1, got %s", cfg.Network)
	}
}

func TestDefaultConfig(t *testing.T) {
	cfg := DefaultConfig()

	if cfg.PricePerRequest != 100 {
		t.Errorf("expected default price 100, got %d", cfg.PricePerRequest)
	}
	if cfg.Scheme != "v1-eip155-exact" {
		t.Errorf("expected scheme v1-eip155-exact, got %s", cfg.Scheme)
	}
	if cfg.ExpirySeconds != 3600 {
		t.Errorf("expected expiry 3600, got %d", cfg.ExpirySeconds)
	}
}

func TestCreatePaymentRequired(t *testing.T) {
	cfg := Config{
		PricePerRequest: 100,
		Network:         "eip155:1",
		PaymentToken:    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
		Recipient:       "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
	}

	pr := CreatePaymentRequired(cfg, "/api/data")

	if pr.Version != 1 {
		t.Errorf("expected version 1, got %d", pr.Version)
	}
	if pr.Scheme != "v1-eip155-exact" {
		t.Errorf("expected scheme, got %s", pr.Scheme)
	}
	if pr.Network != "eip155:1" {
		t.Errorf("expected network, got %s", pr.Network)
	}
	if pr.MaxAmount != "100" {
		t.Errorf("expected max_amount 100, got %s", pr.MaxAmount)
	}
	if pr.Recipient != "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E" {
		t.Errorf("unexpected recipient: %s", pr.Recipient)
	}
	if pr.Expiry == 0 {
		t.Error("expected expiry to be set")
	}
	if pr.Salt == "" {
		t.Error("expected salt to be set")
	}
	if len(pr.Instructions) != 1 {
		t.Errorf("expected 1 instruction, got %d", len(pr.Instructions))
	}
}

func TestCreatePaymentRequiredSolana(t *testing.T) {
	cfg := Config{
		PricePerRequest: 50,
		Scheme:          "v1-solana-exact",
		Network:         "solana:101",
		PaymentToken:    "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
		Recipient:       "SolanaRecipient123",
	}

	pr := CreatePaymentRequired(cfg, "/api/solana")

	if pr.Scheme != "v1-solana-exact" {
		t.Errorf("expected solana scheme, got %s", pr.Scheme)
	}
	if pr.Network != "solana:101" {
		t.Errorf("expected solana network, got %s", pr.Network)
	}
}

func TestPaymentRequiredExpiry(t *testing.T) {
	cfg := Config{
		PricePerRequest: 100,
		Network:         "eip155:1",
		PaymentToken:    "0xABC",
		Recipient:       "0x123",
		ExpirySeconds:   7200,
	}

	before := time.Now().Unix()
	pr := CreatePaymentRequired(cfg, "/test")
	after := time.Now().Unix()

	if pr.Expiry < before+7200 || pr.Expiry > after+7200 {
		t.Errorf("expiry not in expected range: %d", pr.Expiry)
	}
}

func TestParsePaymentToken(t *testing.T) {
	gin.SetMode(gin.TestMode)

	token := PaymentToken{
		Signature:  "0xabc123",
		Sender:     "0xsender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}

	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest("GET", "/test", nil)
	c.Request.Header.Set("X-Payment-Token", encoded)

	result := ParsePaymentToken(c)

	if result == nil {
		t.Fatal("expected token, got nil")
	}
	if result.Signature != "0xabc123" {
		t.Errorf("expected signature, got %s", result.Signature)
	}
}

func TestParsePaymentTokenEmpty(t *testing.T) {
	gin.SetMode(gin.TestMode)

	w := httptest.NewRecorder()
	c, _ := gin.CreateTestContext(w)
	c.Request = httptest.NewRequest("GET", "/test", nil)

	result := ParsePaymentToken(c)

	if result != nil {
		t.Errorf("expected nil, got %v", result)
	}
}

func TestPaymentTokenIsValid(t *testing.T) {
	validToken := &PaymentToken{
		Signature:  "sig",
		Sender:     "sender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}

	if !validToken.IsValid() {
		t.Error("expected valid token to return true")
	}

	invalidToken := &PaymentToken{}
	if invalidToken.IsValid() {
		t.Error("expected invalid token to return false")
	}

	nilToken := (*PaymentToken)(nil)
	if nilToken.IsValid() {
		t.Error("expected nil token to return false")
	}
}

func TestMiddlewareWithoutPayment(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := DefaultConfig()
	cfg.Network = "eip155:1"
	cfg.PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
	cfg.Recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"

	router := gin.New()
	router.Use(Middleware(cfg))
	router.GET("/api/data", func(c *gin.Context) {
		c.JSON(200, gin.H{"data": "success"})
	})

	w := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "/api/data", nil)
	router.ServeHTTP(w, req)

	if w.Code != http.StatusPaymentRequired {
		t.Errorf("expected 402, got %d", w.Code)
	}

	var pr PaymentRequired
	err := json.Unmarshal(w.Body.Bytes(), &pr)
	if err != nil {
		t.Fatalf("failed to parse response: %v", err)
	}
	if pr.Version != 1 {
		t.Errorf("expected version 1, got %d", pr.Version)
	}
}

func TestMiddlewareWithValidToken(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := DefaultConfig()
	cfg.Network = "eip155:1"
	cfg.PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
	cfg.Recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"

	router := gin.New()
	router.Use(Middleware(cfg))
	router.GET("/api/data", func(c *gin.Context) {
		c.JSON(200, gin.H{"data": "success"})
	})

	token := PaymentToken{
		Signature:  "0xvalid",
		Sender:     "0xsender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}
	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	w := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "/api/data", nil)
	req.Header.Set("X-Payment-Token", encoded)
	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestMiddlewareExcludedPaths(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := DefaultConfig()

	router := gin.New()
	router.Use(Middleware(cfg))
	router.GET("/docs", func(c *gin.Context) {
		c.JSON(200, gin.H{"docs": "ok"})
	})

	w := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "/docs", nil)
	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200 for excluded path, got %d", w.Code)
	}
}

func TestRequirePayment(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := DefaultConfig()
	cfg.Network = "eip155:1"
	cfg.PaymentToken = "0xABC"
	cfg.Recipient = "0x123"

	router := gin.New()
	router.GET("/premium", RequirePayment(cfg), func(c *gin.Context) {
		c.JSON(200, gin.H{"premium": "content"})
	})

	// Without token
	w := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "/premium", nil)
	router.ServeHTTP(w, req)

	if w.Code != http.StatusPaymentRequired {
		t.Errorf("expected 402, got %d", w.Code)
	}

	// With valid token
	token := PaymentToken{
		Signature:  "sig",
		Sender:     "sender",
		Amount:     "100",
		ValidAfter: time.Now().Unix(),
	}
	tokenJSON, _ := json.Marshal(token)
	encoded := base64.StdEncoding.EncodeToString(tokenJSON)

	w2 := httptest.NewRecorder()
	req2, _ := http.NewRequest("GET", "/premium", nil)
	req2.Header.Set("X-Payment-Token", encoded)
	router.ServeHTTP(w2, req2)

	if w2.Code != http.StatusOK {
		t.Errorf("expected 200 with token, got %d", w2.Code)
	}
}
