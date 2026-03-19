package com.x402;

import jakarta.ws.rs.container.*;
import jakarta.ws.rs.core.*;
import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import org.eclipse.microprofile.config.inject.ConfigProperty;

import java.util.*;

@ApplicationScoped
@Provider
public class X402Interceptor implements ContainerRequestFilter {

    @ConfigProperty(name = "x402.price-per-request", defaultValue = "100")
    int pricePerRequest;

    @ConfigProperty(name = "x402.network", defaultValue = "eip155:1")
    String network;

    @ConfigProperty(name = "x402.payment-token")
    String paymentToken;

    @ConfigProperty(name = "x402.recipient")
    String recipient;

    @ConfigProperty(name = "x402.description", defaultValue = "API access")
    String description;

    @ConfigProperty(name = "x402.facilitator-url", defaultValue = "https://facilitator.x402.rs")
    String facilitatorUrl;

    @Context
    UriInfo uriInfo;

    @Override
    public void filter(ContainerRequestContext ctx) {
        String path = uriInfo.getPath();
        
        if (shouldExclude(path)) {
            return;
        }

        String tokenHeader = ctx.getHeaderString("X-Payment-Token");
        
        if (tokenHeader != null && isValidToken(tokenHeader)) {
            return;
        }

        ctx.abortWith(createPaymentRequired(path));
    }

    private boolean shouldExclude(String path) {
        List<String> excludePaths = Arrays.asList("/docs", "/swagger-ui", "/openapi", "/health");
        return excludePaths.stream().anyMatch(path::startsWith);
    }

    private boolean isValidToken(String tokenHeader) {
        try {
            String decoded = new String(Base64.getDecoder().decode(tokenHeader));
            return decoded.contains("\"signature\"") && decoded.contains("\"sender\"");
        } catch (Exception e) {
            return false;
        }
    }

    private Response createPaymentRequired(String path) {
        long now = System.currentTimeMillis() / 1000;
        long expiry = now + 3600;
        
        String salt = Integer.toHexString(Objects.hash(path, now));

        Map<String, Object> instruction = new HashMap<>();
        instruction.put("protocol", "https");
        instruction.put("method", "POST");
        instruction.put("url", facilitatorUrl + "/v1/payment");
        
        Map<String, String> headers = new HashMap<>();
        headers.put("Content-Type", "application/json");
        instruction.put("headers", headers);
        
        Map<String, Object> body = new HashMap<>();
        body.put("network", network);
        body.put("payment_token", paymentToken);
        body.put("amount", String.valueOf(pricePerRequest));
        body.put("recipient", recipient);
        body.put("valid_after", now);
        body.put("salt", salt);
        instruction.put("body", body);

        List<Map<String, Object>> instructions = new ArrayList<>();
        instructions.add(instruction);

        Map<String, Object> pr = new LinkedHashMap<>();
        pr.put("version", 1);
        pr.put("scheme", "v1-eip155-exact");
        pr.put("network", network);
        pr.put("payment_token", paymentToken);
        pr.put("max_amount", String.valueOf(pricePerRequest));
        pr.put("recipient", recipient);
        pr.put("description", description);
        pr.put("expiry", expiry);
        pr.put("salt", salt);
        pr.put("instructions", instructions);

        return Response.status(402).entity(pr).build();
    }
}
