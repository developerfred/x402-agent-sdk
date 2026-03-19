# x402 Rails Tests
# Run with: ruby -Ilib:test test/x402_rails_test.rb

require 'minitest/autorun'
require 'json'
require 'base64'
require 'time'

require_relative 'lib/x402_rails'

class X402ConfigurationTest < Minitest::Test
  def test_default_configuration
    config = X402Rails::Configuration.new
    
    assert_equal 100, config.price_per_request
    assert_equal 'v1-eip155-exact', config.scheme
    assert_equal 'eip155:1', config.network
    assert_equal 'API access', config.description
    assert_equal 3600, config.expiry_seconds
    assert_includes config.exclude_paths, '/docs'
  end

  def test_custom_configuration
    config = X402Rails::Configuration.new
    config.price_per_request = 500
    config.scheme = 'v1-solana-exact'
    config.network = 'solana:101'
    config.payment_token = 'EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ'
    config.recipient = 'SolanaRecipient'
    config.description = 'Premium Access'
    config.expiry_seconds = 7200
    
    assert_equal 500, config.price_per_request
    assert_equal 'v1-solana-exact', config.scheme
    assert_equal 'solana:101', config.network
    assert_equal 'Premium Access', config.description
  end
end

class PaymentRequiredTest < Minitest::Test
  def setup
    @config = X402Rails::Configuration.new
    @config.payment_token = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
    @config.recipient = '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E'
  end

  def test_create_payment_required
    pr = X402Rails::PaymentRequired.new(@config, '/api/data')
    
    assert_equal 1, pr.version
    assert_equal 'v1-eip155-exact', pr.scheme
    assert_equal 'eip155:1', pr.network
    assert_equal '100', pr.max_amount
    assert_equal '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E', pr.recipient
    assert pr.expiry > Time.now.to_i
    assert pr.salt.length > 0
    assert pr.instructions.is_a?(Array)
    assert_equal 1, pr.instructions.length
  end

  def test_create_solana_payment
    @config.scheme = 'v1-solana-exact'
    @config.network = 'solana:101'
    @config.payment_token = 'EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ'
    
    pr = X402Rails::PaymentRequired.new(@config, '/api/solana')
    
    assert_equal 'v1-solana-exact', pr.scheme
    assert_equal 'solana:101', pr.network
  end

  def test_payment_expiry
    @config.expiry_seconds = 7200
    
    before = Time.now.to_i
    pr = X402Rails::PaymentRequired.new(@config, '/test')
    after = Time.now.to_i
    
    assert pr.expiry >= before + 7200
    assert pr.expiry <= after + 7200
  end

  def test_payment_required_instructions
    @config.facilitator_url = 'https://facilitator.x402.rs'
    
    pr = X402Rails::PaymentRequired.new(@config, '/api/data')
    
    instruction = pr.instructions.first
    assert_equal 'https', instruction[:protocol]
    assert_equal 'POST', instruction[:method]
    assert_equal 'https://facilitator.x402.rs/v1/payment', instruction[:url]
    assert_equal 'application/json', instruction[:headers]['Content-Type']
    assert_equal 'eip155:1', instruction[:body][:network]
  end

  def test_to_h
    pr = X402Rails::PaymentRequired.new(@config, '/api/data')
    hash = pr.to_h
    
    assert_equal 1, hash[:version]
    assert_equal 'v1-eip155-exact', hash[:scheme]
    assert_equal 'eip155:1', hash[:network]
    assert hash.key?(:expiry)
    assert hash.key?(:salt)
    assert hash.key?(:instructions)
  end
end

class X402MiddlewareTest < Minitest::Test
  def setup
    @config = X402Rails::Configuration.new
    @config.payment_token = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
    @config.recipient = '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E'
    @middleware = X402Rails::Middleware.new(@app, @config)
  end

  def test_middleware_returns_402_without_token
    # Simulate request without payment token
    env = {
      'PATH_INFO' => '/api/data',
      'HTTP_X_PAYMENT_TOKEN' => nil
    }
    
    status, headers, body = @middleware.call(env)
    
    assert_equal 402, status
    assert_equal 'application/json', headers['Content-Type']
    
    response = JSON.parse(body.first)
    assert_equal 1, response['version']
    assert response.key?('payment_token')
  end

  def test_middleware_returns_200_with_valid_token
    token_data = {
      signature: '0xabc123',
      sender: '0xsender',
      amount: '100',
      valid_after: Time.now.to_i
    }
    encoded_token = Base64.encode64(JSON.generate(token_data))
    
    # Create a mock app that returns 200
    mock_app = lambda { |_env| [200, {'Content-Type' => 'application/json'}, ['{"data":"success"}']] }
    middleware = X402Rails::Middleware.new(mock_app, @config)
    
    env = {
      'PATH_INFO' => '/api/data',
      'HTTP_X_PAYMENT_TOKEN' => encoded_token
    }
    
    status, _headers, _body = middleware.call(env)
    
    assert_equal 200, status
  end

  def test_middleware_excludes_paths
    mock_app = lambda { |_env| [200, {}, []] }
    middleware = X402Rails::Middleware.new(mock_app, @config)
    
    env = { 'PATH_INFO' => '/docs' }
    
    status, _headers, _body = middleware.call(env)
    
    assert_equal 200, status
  end

  def test_middleware_passes_through_invalid_token
    env = {
      'PATH_INFO' => '/api/data',
      'HTTP_X_PAYMENT_TOKEN' => 'invalid-base64!!!'
    }
    
    status, _headers, _body = @middleware.call(env)
    
    assert_equal 402, status
  end
end

class ConfigurationDefaultsTest < Minitest::Test
  def test_exclude_paths_defaults
    config = X402Rails::Configuration.new
    
    assert_includes config.exclude_paths, '/docs'
    assert_includes config.exclude_paths, '/static'
  end
end
