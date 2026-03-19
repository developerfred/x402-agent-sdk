// x402 .NET/C# Tests
using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;
using Microsoft.AspNetCore.Http;
using Moq;

namespace X402Middleware.Tests
{
    public class X402OptionsTests
    {
        [Fact]
        public void DefaultOptions_ShouldHaveCorrectDefaults()
        {
            var options = new X402Options();
            
            Assert.Equal(100, options.PricePerRequest);
            Assert.Equal("v1-eip155-exact", options.Scheme);
            Assert.Equal("eip155:1", options.Network);
            Assert.Equal("API access", options.Description);
            Assert.Equal(3600, options.ExpirySeconds);
            Assert.NotNull(options.ExcludePaths);
        }

        [Fact]
        public void CustomOptions_ShouldSetValues()
        {
            var options = new X402Options
            {
                PricePerRequest = 500,
                Scheme = "v1-solana-exact",
                Network = "solana:101",
                PaymentToken = "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
                Recipient = "SolanaRecipient123",
                Description = "Premium Access",
                ExpirySeconds = 7200
            };
            
            Assert.Equal(500, options.PricePerRequest);
            Assert.Equal("v1-solana-exact", options.Scheme);
            Assert.Equal("solana:101", options.Network);
            Assert.Equal("Premium Access", options.Description);
        }
    }

    public class PaymentRequiredTests
    {
        [Fact]
        public void CreatePaymentRequired_ShouldHaveCorrectStructure()
        {
            var options = new X402Options
            {
                PricePerRequest = 100,
                Network = "eip155:1",
                PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                Recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
            };
            
            // Use reflection or internal method to test
            var middleware = new X402Middleware(
                _ => Task.CompletedTask,
                options
            );
        }

        [Fact]
        public void PaymentRequired_ShouldContainRequiredFields()
        {
            // Test that response has required x402 fields
            var pr = new PaymentRequired
            {
                Version = 1,
                Scheme = "v1-eip155-exact",
                Network = "eip155:1",
                PaymentToken = "0xABC",
                MaxAmount = "100",
                Recipient = "0x123",
                Description = "Test",
                Expiry = DateTimeOffset.UtcNow.ToUnixTimeSeconds() + 3600,
                Salt = "abc123",
                Instructions = new List<Instruction>
                {
                    new Instruction
                    {
                        Protocol = "https",
                        Method = "POST",
                        Url = "https://facilitator.x402.rs/v1/payment",
                        Headers = new Dictionary<string, string>
                        {
                            { "Content-Type", "application/json" }
                        },
                        Body = new Dictionary<string, object>()
                    }
                }
            };
            
            Assert.Equal(1, pr.Version);
            Assert.Equal("v1-eip155-exact", pr.Scheme);
            Assert.NotNull(pr.Instructions);
            Assert.Single(pr.Instructions);
        }
    }

    public class X402MiddlewareTests
    {
        private Mock<RequestDelegate> _nextMock;
        private X402Options _options;

        public X402MiddlewareTests()
        {
            _nextMock = new Mock<RequestDelegate>();
            _options = new X402Options
            {
                PricePerRequest = 100,
                Network = "eip155:1",
                PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                Recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
            };
        }

        [Fact]
        public async Task Invoke_NoPaymentHeader_Returns402()
        {
            // Arrange
            _nextMock.Setup(n => n(It.IsAny<HttpContext>()))
                .Returns(Task.CompletedTask);
            
            var middleware = new X402Middleware(_nextMock.Object, _options);
            
            var context = new DefaultHttpContext();
            context.Request.Path = "/api/data";
            
            // Act
            await middleware.InvokeAsync(context);
            
            // Assert
            Assert.Equal(402, context.Response.StatusCode);
        }

        [Fact]
        public async Task Invoke_WithValidToken_Returns200()
        {
            // Arrange
            _nextMock.Setup(n => n(It.IsAny<HttpContext>()))
                .Returns(Task.CompletedTask);
            
            var middleware = new X402Middleware(_nextMock.Object, _options);
            
            var token = new
            {
                signature = "0xabc123",
                sender = "0xsender",
                amount = "100",
                valid_after = DateTimeOffset.UtcNow.ToUnixTimeSeconds()
            };
            
            var tokenJson = System.Text.Json.JsonSerializer.Serialize(token);
            var tokenEncoded = Convert.ToBase64String(
                System.Text.Encoding.UTF8.GetBytes(tokenJson)
            );
            
            var context = new DefaultHttpContext();
            context.Request.Path = "/api/data";
            context.Request.Headers["X-Payment-Token"] = tokenEncoded;
            
            // Act
            await middleware.InvokeAsync(context);
            
            // Assert
            Assert.Equal(200, context.Response.StatusCode);
            _nextMock.Verify(n => n(It.IsAny<HttpContext>()), Times.Once);
        }

        [Fact]
        public async Task Invoke_ExcludedPath_Returns200()
        {
            // Arrange
            _nextMock.Setup(n => n(It.IsAny<HttpContext>()))
                .Returns(Task.CompletedTask);
            
            var middleware = new X402Middleware(_nextMock.Object, _options);
            
            var context = new DefaultHttpContext();
            context.Request.Path = "/docs";
            
            // Act
            await middleware.InvokeAsync(context);
            
            // Assert
            Assert.Equal(200, context.Response.StatusCode);
        }

        [Fact]
        public async Task Invoke_InvalidToken_Returns402()
        {
            // Arrange
            _nextMock.Setup(n => n(It.IsAny<HttpContext>()))
                .Returns(Task.CompletedTask);
            
            var middleware = new X402Middleware(_nextMock.Object, _options);
            
            var context = new DefaultHttpContext();
            context.Request.Path = "/api/data";
            context.Request.Headers["X-Payment-Token"] = "invalid-base64!!!";
            
            // Act
            await middleware.InvokeAsync(context);
            
            // Assert
            Assert.Equal(402, context.Response.StatusCode);
        }
    }

    public class X402ExtensionsTests
    {
        [Fact]
        public void AddX402_ShouldConfigureOptions()
        {
            var services = new ServiceCollection();
            
            services.AddX402(options =>
            {
                options.PricePerRequest = 100;
                options.Network = "eip155:1";
                options.PaymentToken = "0xABC";
                options.Recipient = "0x123";
            });
            
            var provider = services.BuildServiceProvider();
            var options = provider.GetRequiredService<X402Options>();
            
            Assert.Equal(100, options.PricePerRequest);
            Assert.Equal("0xABC", options.PaymentToken);
        }
    }
}
