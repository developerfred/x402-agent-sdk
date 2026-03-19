package com.x402;

import java.time.Instant;
import java.util.*;

class X402Config {
    private int pricePerRequest = 100;
    private String scheme = "v1-eip155-exact";
    private String network = "eip155:1";
    private String paymentToken;
    private String recipient;
    private String description = "API access";
    private int expirySeconds = 3600;
    private String facilitatorUrl = "https://facilitator.x402.rs";

    public X402Config() {}

    public X402Config(int pricePerRequest, String network, String paymentToken, String recipient) {
        this.pricePerRequest = pricePerRequest;
        this.network = network;
        this.paymentToken = paymentToken;
        this.recipient = recipient;
    }

    public int getPricePerRequest() { return pricePerRequest; }
    public void setPricePerRequest(int pricePerRequest) { this.pricePerRequest = pricePerRequest; }
    public String getScheme() { return scheme; }
    public void setScheme(String scheme) { this.scheme = scheme; }
    public String getNetwork() { return network; }
    public void setNetwork(String network) { this.network = network; }
    public String getPaymentToken() { return paymentToken; }
    public void setPaymentToken(String paymentToken) { this.paymentToken = paymentToken; }
    public String getRecipient() { return recipient; }
    public void setRecipient(String recipient) { this.recipient = recipient; }
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
    public int getExpirySeconds() { return expirySeconds; }
    public void setExpirySeconds(int expirySeconds) { this.expirySeconds = expirySeconds; }
    public String getFacilitatorUrl() { return facilitatorUrl; }
    public void setFacilitatorUrl(String facilitatorUrl) { this.facilitatorUrl = facilitatorUrl; }
}

class PaymentRequired {
    private int version = 1;
    private String scheme;
    private String network;
    private String paymentToken;
    private String maxAmount;
    private String recipient;
    private String description;
    private long expiry;
    private String salt;
    private List<Map<String, Object>> instructions;

    public static PaymentRequired create(X402Config config, String path) {
        long now = Instant.now().getEpochSecond();
        long expiry = now + config.getExpirySeconds();
        
        String salt = Integer.toHexString(
            Objects.hash(path, now)
        );

        Map<String, Object> instruction = new HashMap<>();
        instruction.put("protocol", "https");
        instruction.put("method", "POST");
        instruction.put("url", config.getFacilitatorUrl() + "/v1/payment");
        
        Map<String, String> headers = new HashMap<>();
        headers.put("Content-Type", "application/json");
        instruction.put("headers", headers);
        
        Map<String, Object> body = new HashMap<>();
        body.put("network", config.getNetwork());
        body.put("payment_token", config.getPaymentToken());
        body.put("amount", String.valueOf(config.getPricePerRequest()));
        body.put("recipient", config.getRecipient());
        body.put("valid_after", now);
        body.put("salt", salt);
        instruction.put("body", body);

        List<Map<String, Object>> instructions = new ArrayList<>();
        instructions.add(instruction);

        PaymentRequired pr = new PaymentRequired();
        pr.setVersion(1);
        pr.setScheme(config.getScheme());
        pr.setNetwork(config.getNetwork());
        pr.setPaymentToken(config.getPaymentToken());
        pr.setMaxAmount(String.valueOf(config.getPricePerRequest()));
        pr.setRecipient(config.getRecipient());
        pr.setDescription(config.getDescription());
        pr.setExpiry(expiry);
        pr.setSalt(salt);
        pr.setInstructions(instructions);

        return pr;
    }

    public int getVersion() { return version; }
    public void setVersion(int version) { this.version = version; }
    public String getScheme() { return scheme; }
    public void setScheme(String scheme) { this.scheme = scheme; }
    public String getNetwork() { return network; }
    public void setNetwork(String network) { this.network = network; }
    public String getPaymentToken() { return paymentToken; }
    public void setPaymentToken(String paymentToken) { this.paymentToken = paymentToken; }
    public String getMaxAmount() { return maxAmount; }
    public void setMaxAmount(String maxAmount) { this.maxAmount = maxAmount; }
    public String getRecipient() { return recipient; }
    public void setRecipient(String recipient) { this.recipient = recipient; }
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
    public long getExpiry() { return expiry; }
    public void setExpiry(long expiry) { this.expiry = expiry; }
    public String getSalt() { return salt; }
    public void setSalt(String salt) { this.salt = salt; }
    public List<Map<String, Object>> getInstructions() { return instructions; }
    public void setInstructions(List<Map<String, Object>> instructions) { this.instructions = instructions; }
}

class PaymentToken {
    private String signature;
    private String sender;
    private String amount;
    private long validAfter;

    public boolean isValid() {
        return signature != null && !signature.isEmpty()
            && sender != null && !sender.isEmpty()
            && amount != null && !amount.isEmpty()
            && validAfter > 0;
    }

    public String getSignature() { return signature; }
    public void setSignature(String signature) { this.signature = signature; }
    public String getSender() { return sender; }
    public void setSender(String sender) { this.sender = sender; }
    public String getAmount() { return amount; }
    public void setAmount(String amount) { this.amount = amount; }
    public long getValidAfter() { return validAfter; }
    public void setValidAfter(long validAfter) { this.validAfter = validAfter; }
}
