using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;

namespace X402Middleware
{
    public class X402Options
    {
        public long PricePerRequest { get; set; } = 100;
        public string Scheme { get; set; } = "v1-eip155-exact";
        public string Network { get; set; } = "eip155:1";
        public string PaymentToken { get; set; } = "";
        public string Recipient { get; set; } = "";
        public string Description { get; set; } = "API access";
        public int ExpirySeconds { get; set; } = 3600;
        public string FacilitatorUrl { get; set; } = "https://facilitator.x402.rs";
        public List<string> ExcludePaths { get; set; } = new List<string> { "/docs", "/swagger", "/static" };
    }

    public class PaymentRequired
    {
        public int Version { get; set; } = 1;
        public string Scheme { get; set; }
        public string Network { get; set; }
        public string PaymentToken { get; set; }
        public string MaxAmount { get; set; }
        public string Recipient { get; set; }
        public string Description { get; set; }
        public long Expiry { get; set; }
        public string Salt { get; set; }
        public List<Instruction> Instructions { get; set; }
    }

    public class Instruction
    {
        public string Protocol { get; set; }
        public string Method { get; set; }
        public string Url { get; set; }
        public Dictionary<string, string> Headers { get; set; }
        public Dictionary<string, object> Body { get; set; }
    }

    public class X402Middleware
    {
        private readonly RequestDelegate _next;
        private readonly X402Options _options;

        public X402Middleware(RequestDelegate next, X402Options options)
        {
            _next = next;
            _options = options;
        }

        public async Task InvokeAsync(HttpContext context)
        {
            var path = context.Request.Path.Value;

            if (ShouldExclude(path))
            {
                await _next(context);
                return;
            }

            var tokenHeader = context.Request.Headers["X-Payment-Token"].ToString();

            if (!string.IsNullOrEmpty(tokenHeader) && IsValidToken(tokenHeader))
            {
                await _next(context);
                return;
            }

            var paymentRequired = CreatePaymentRequired(path);
            
            context.Response.StatusCode = 402;
            context.Response.ContentType = "application/json";
            
            await context.Response.WriteAsJsonAsync(paymentRequired);
        }

        private bool ShouldExclude(string path)
        {
            foreach (var excludePath in _options.ExcludePaths)
            {
                if (path.StartsWith(excludePath, StringComparison.OrdinalIgnoreCase))
                    return true;
            }
            return false;
        }

        private bool IsValidToken(string tokenHeader)
        {
            try
            {
                var decoded = System.Text.Encoding.UTF8.GetString(
                    Convert.FromBase64String(tokenHeader));
                return decoded.Contains("\"signature\"") && decoded.Contains("\"sender\"");
            }
            catch
            {
                return false;
            }
        }

        private PaymentRequired CreatePaymentRequired(string path)
        {
            var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            var expiry = now + _options.ExpirySeconds;
            
            var salt = Convert.ToHexString(
                System.Security.Cryptography.MD5.HashData(
                    System.Text.Encoding.UTF8.GetBytes($"{path}:{now}")))[..16];

            var body = new Dictionary<string, object>
            {
                { "network", _options.Network },
                { "payment_token", _options.PaymentToken },
                { "amount", _options.PricePerRequest.ToString() },
                { "recipient", _options.Recipient },
                { "valid_after", now },
                { "salt", salt }
            };

            return new PaymentRequired
            {
                Version = 1,
                Scheme = _options.Scheme,
                Network = _options.Network,
                PaymentToken = _options.PaymentToken,
                MaxAmount = _options.PricePerRequest.ToString(),
                Recipient = _options.Recipient,
                Description = _options.Description,
                Expiry = expiry,
                Salt = salt,
                Instructions = new List<Instruction>
                {
                    new Instruction
                    {
                        Protocol = "https",
                        Method = "POST",
                        Url = $"{_options.FacilitatorUrl}/v1/payment",
                        Headers = new Dictionary<string, string>
                        {
                            { "Content-Type", "application/json" }
                        },
                        Body = body
                    }
                }
            };
        }
    }

    public static class X402Extensions
    {
        public static IServiceCollection AddX402(this IServiceCollection services, Action<X402Options> configure)
        {
            var options = new X402Options();
            configure(options);
            services.AddSingleton(options);
            return services;
        }

        public static IApplicationBuilder UseX402(this IApplicationBuilder builder, X402Options options)
        {
            return builder.UseMiddleware<X402Middleware>(options);
        }
    }
}
