package com.x402;

import jakarta.servlet.*;
import jakarta.servlet.http.*;
import org.springframework.boot.web.servlet.FilterRegistrationBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;

import java.io.IOException;
import java.util.*;

@Configuration
public class X402Filter implements Filter {

    private X402Config config;

    public X402Filter(X402Config config) {
        this.config = config;
    }

    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {
        
        HttpServletRequest httpRequest = (HttpServletRequest) request;
        HttpServletResponse httpResponse = (HttpServletResponse) response;
        
        String path = httpRequest.getRequestURI();
        
        if (shouldExclude(path)) {
            chain.doFilter(request, response);
            return;
        }

        String tokenHeader = httpRequest.getHeader("X-Payment-Token");
        
        if (tokenHeader != null && isValidToken(tokenHeader)) {
            chain.doFilter(request, response);
            return;
        }

        httpResponse.setStatus(402);
        httpResponse.setContentType("application/json");
        
        PaymentRequired pr = PaymentRequired.create(config, path);
        
        String json = toJson(pr);
        response.getWriter().write(json);
    }

    private boolean shouldExclude(String path) {
        List<String> excludePaths = Arrays.asList("/docs", "/swagger-ui", "/v3/api-docs", "/static");
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

    private String toJson(PaymentRequired pr) {
        StringBuilder sb = new StringBuilder();
        sb.append("{");
        sb.append("\"version\":").append(pr.getVersion()).append(",");
        sb.append("\"scheme\":\"").append(pr.getScheme()).append("\",");
        sb.append("\"network\":\"").append(pr.getNetwork()).append("\",");
        sb.append("\"payment_token\":\"").append(pr.getPaymentToken()).append("\",");
        sb.append("\"max_amount\":\"").append(pr.getMaxAmount()).append("\",");
        sb.append("\"recipient\":\"").append(pr.getRecipient()).append("\",");
        sb.append("\"description\":\"").append(pr.getDescription()).append("\",");
        sb.append("\"expiry\":").append(pr.getExpiry()).append(",");
        sb.append("\"salt\":\"").append(pr.getSalt()).append("\"");
        sb.append("}");
        return sb.toString();
    }

    @Bean
    public FilterRegistrationBean<X402Filter> x402FilterRegistration() {
        FilterRegistrationBean<X402Filter> registration = new FilterRegistrationBean<>();
        registration.setFilter(new X402Filter(config));
        registration.addUrlPatterns("/*");
        registration.setOrder(Ordered.HIGHEST_PRECEDENCE);
        registration.setName("X402Filter");
        return registration;
    }
}
