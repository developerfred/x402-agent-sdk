package fiber

import (
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"time"

	"github.com/gofiber/fiber/v2"
)

type Config struct {
	PricePerRequest int    `json:"price_per_request"`
	Scheme          string `json:"scheme"`
	Network         string `json:"network"`
	PaymentToken    string `json:"payment_token"`
	Recipient       string `json:"recipient"`
	Description     string `json:"description"`
	ExpirySeconds   int    `json:"expiry_seconds"`
	FacilitatorURL  string `json:"facilitator_url"`
}

type PaymentRequired struct {
	Version      int           `json:"version"`
	Scheme       string        `json:"scheme"`
	Network      string        `json:"network"`
	PaymentToken string        `json:"payment_token"`
	MaxAmount    string        `json:"max_amount"`
	Recipient    string        `json:"recipient"`
	Description  string        `json:"description"`
	Expiry       int64         `json:"expiry"`
	Salt         string        `json:"salt"`
	Instructions []Instruction `json:"instructions"`
}

type Instruction struct {
	Protocol string                 `json:"protocol"`
	Method   string                 `json:"method"`
	URL      string                 `json:"url"`
	Headers  map[string]string      `json:"headers"`
	Body     map[string]interface{} `json:"body"`
}

type PaymentToken struct {
	Signature  string `json:"signature"`
	Sender     string `json:"sender"`
	Amount     string `json:"amount"`
	ValidAfter int64  `json:"valid_after"`
}

func DefaultConfig() Config {
	return Config{
		PricePerRequest: 100,
		Scheme:          "v1-eip155-exact",
		Network:         "eip155:1",
		Description:     "API access",
		ExpirySeconds:   3600,
		FacilitatorURL:  "https://facilitator.x402.rs",
	}
}

func CreatePaymentRequired(config Config, path string) PaymentRequired {
	now := time.Now().Unix()
	expiry := now + int64(config.ExpirySeconds)
	salt := fmt.Sprintf("%x", sha256.Sum256([]byte(fmt.Sprintf("%s:%d", path, now))))[:16]

	defaultConfig := DefaultConfig()
	if config.Scheme == "" {
		config.Scheme = defaultConfig.Scheme
	}
	if config.ExpirySeconds == 0 {
		config.ExpirySeconds = defaultConfig.ExpirySeconds
	}
	if config.FacilitatorURL == "" {
		config.FacilitatorURL = defaultConfig.FacilitatorURL
	}

	return PaymentRequired{
		Version:      1,
		Scheme:       config.Scheme,
		Network:      config.Network,
		PaymentToken: config.PaymentToken,
		MaxAmount:    fmt.Sprintf("%d", config.PricePerRequest),
		Recipient:    config.Recipient,
		Description:  config.Description,
		Expiry:       expiry,
		Salt:         salt,
		Instructions: []Instruction{
			{
				Protocol: "https",
				Method:   "POST",
				URL:      config.FacilitatorURL + "/v1/payment",
				Headers:  map[string]string{"Content-Type": "application/json"},
				Body: map[string]interface{}{
					"network":       config.Network,
					"payment_token": config.PaymentToken,
					"amount":        fmt.Sprintf("%d", config.PricePerRequest),
					"recipient":     config.Recipient,
					"valid_after":   now,
					"salt":          salt,
				},
			},
		},
	}
}

func ParsePaymentToken(c *fiber.Ctx) *PaymentToken {
	tokenHeader := c.Get("X-Payment-Token")
	if tokenHeader == "" {
		return nil
	}

	decoded, err := base64.StdEncoding.DecodeString(tokenHeader)
	if err != nil {
		return nil
	}

	var token PaymentToken
	if err := json.Unmarshal(decoded, &token); err != nil {
		return nil
	}

	return &token
}

func (p *PaymentToken) IsValid() bool {
	if p == nil {
		return false
	}
	return p.Signature != "" && p.Sender != "" && p.Amount != "" && p.ValidAfter > 0
}

func Middleware(config Config) fiber.Handler {
	defaultConfig := DefaultConfig()
	if config.PricePerRequest > 0 {
		defaultConfig.PricePerRequest = config.PricePerRequest
	}
	if config.Network != "" {
		defaultConfig.Network = config.Network
	}
	if config.PaymentToken != "" {
		defaultConfig.PaymentToken = config.PaymentToken
	}
	if config.Recipient != "" {
		defaultConfig.Recipient = config.Recipient
	}

	excludePaths := []string{"/docs", "/swagger/", "/static/"}

	return func(c *fiber.Ctx) error {
		path := c.Path()

		excluded := false
		for _, p := range excludePaths {
			if len(path) >= len(p) && path[:len(p)] == p {
				excluded = true
				break
			}
		}

		if excluded {
			return c.Next()
		}

		token := ParsePaymentToken(c)

		if token != nil && token.IsValid() {
			return c.Next()
		}

		pr := CreatePaymentRequired(defaultConfig, path)
		return c.Status(fiber.StatusPaymentRequired).JSON(pr)
	}
}

func RequirePayment(config Config) fiber.Handler {
	return func(c *fiber.Ctx) error {
		token := ParsePaymentToken(c)
		if token == nil || !token.IsValid() {
			pr := CreatePaymentRequired(config, c.Path())
			return c.Status(fiber.StatusPaymentRequired).JSON(pr)
		}
		return c.Next()
	}
}
