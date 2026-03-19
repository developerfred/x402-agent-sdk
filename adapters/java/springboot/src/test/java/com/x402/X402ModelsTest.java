// x402 Java Spring Boot Tests
package com.x402;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.BeforeEach;
import static org.junit.jupiter.api.Assertions.*;

import java.util.*;

public class X402ModelsTest {

    @Test
    void testConfigCreation() {
        X402Config config = new X402Config(
            100,
            "eip155:1",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
        );
        
        assertEquals(100, config.getPricePerRequest());
        assertEquals("eip155:1", config.getNetwork());
    }

    @Test
    void testConfigDefaults() {
        X402Config config = new X402Config();
        
        assertEquals(100, config.getPricePerRequest());
        assertEquals("v1-eip155-exact", config.getScheme());
        assertEquals("eip155:1", config.getNetwork());
        assertEquals(3600, config.getExpirySeconds());
    }

    @Test
    void testPaymentRequiredCreation() {
        X402Config config = new X402Config(
            100,
            "eip155:1",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
        );
        
        PaymentRequired pr = PaymentRequired.create(config, "/api/data");
        
        assertEquals(1, pr.getVersion());
        assertEquals("v1-eip155-exact", pr.getScheme());
        assertEquals("eip155:1", pr.getNetwork());
        assertEquals("100", pr.getMaxAmount());
        assertEquals("0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E", pr.getRecipient());
        assertTrue(pr.getExpiry() > 0);
        assertNotNull(pr.getSalt());
        assertNotNull(pr.getInstructions());
        assertEquals(1, pr.getInstructions().size());
    }

    @Test
    void testSolanaPaymentRequired() {
        X402Config config = new X402Config(
            50,
            "solana:101",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            "SolanaRecipient123"
        );
        config.setScheme("v1-solana-exact");
        
        PaymentRequired pr = PaymentRequired.create(config, "/api/solana");
        
        assertEquals("v1-solana-exact", pr.getScheme());
        assertEquals("solana:101", pr.getNetwork());
        assertEquals("50", pr.getMaxAmount());
    }

    @Test
    void testPaymentRequiredExpiry() {
        X402Config config = new X402Config(
            100,
            "eip155:1",
            "0xABC",
            "0x123"
        );
        config.setExpirySeconds(7200);
        
        long before = System.currentTimeMillis() / 1000;
        PaymentRequired pr = PaymentRequired.create(config, "/test");
        long after = System.currentTimeMillis() / 1000;
        
        assertTrue(pr.getExpiry() >= before + 7200);
        assertTrue(pr.getExpiry() <= after + 7200);
    }

    @Test
    void testPaymentRequiredInstructions() {
        X402Config config = new X402Config(
            100,
            "eip155:1",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
        );
        config.setFacilitatorUrl("https://facilitator.x402.rs");
        
        PaymentRequired pr = PaymentRequired.create(config, "/api/data");
        
        Map<String, Object> instruction = pr.getInstructions().get(0);
        assertEquals("https", instruction.get("protocol"));
        assertEquals("POST", instruction.get("method"));
        assertEquals("https://facilitator.x402.rs/v1/payment", instruction.get("url"));
        
        Map<String, Object> body = (Map<String, Object>) instruction.get("body");
        assertEquals("eip155:1", body.get("network"));
        assertEquals("100", body.get("amount"));
    }

    @Test
    void testPaymentTokenValidation() {
        PaymentToken token = new PaymentToken();
        
        // Empty token should be invalid
        assertFalse(token.isValid());
        
        // Token with signature should be valid
        token.setSignature("0xabc123");
        token.setSender("0xsender");
        token.setAmount("100");
        token.setValidAfter(System.currentTimeMillis() / 1000);
        
        assertTrue(token.isValid());
        
        // Token without signature should be invalid
        PaymentToken token2 = new PaymentToken();
        token2.setSender("0xsender");
        token2.setAmount("100");
        token2.setValidAfter(System.currentTimeMillis() / 1000);
        
        assertFalse(token2.isValid());
    }

    @Test
    void testCustomDescription() {
        X402Config config = new X402Config(
            100,
            "eip155:1",
            "0xABC",
            "0x123"
        );
        config.setDescription("Premium API Access");
        
        PaymentRequired pr = PaymentRequired.create(config, "/api/premium");
        
        assertEquals("Premium API Access", pr.getDescription());
    }
}
