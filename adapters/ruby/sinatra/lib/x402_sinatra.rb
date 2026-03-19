require 'base64'
require 'json'
require 'digest'
require 'time'

module X402Sinatra
  class Configuration
    attr_accessor :price_per_request, :scheme, :network, :payment_token, :recipient
    attr_accessor :description, :expiry_seconds, :facilitator_url, :exclude_paths

    def initialize(options = {})
      @price_per_request = options[:price_per_request] || 100
      @scheme = options[:scheme] || 'v1-eip155-exact'
      @network = options[:network] || 'eip155:1'
      @payment_token = options[:payment_token] || ''
      @recipient = options[:recipient] || ''
      @description = options[:description] || 'API access'
      @expiry_seconds = options[:expiry_seconds] || 3600
      @facilitator_url = options[:facilitator_url] || 'https://facilitator.x402.rs'
      @exclude_paths = options[:exclude_paths] || ['/docs', '/static']
    end
  end

  def x402(config = nil)
    @x402_config ||= config || X402Sinatra::Configuration.new
  end

  def x402_middleware(app, config = nil)
    config ||= X402Sinatra::Configuration.new
    
    proc do |env|
      path = env['PATH_INFO']

      if should_exclude?(path, config)
        return app.call(env)
      end

      token_header = env['HTTP_X_PAYMENT_TOKEN']

      if valid_token?(token_header)
        return app.call(env)
      end

      payment_required = create_payment_required(config, path)

      [
        402,
        { 'Content-Type' => 'application/json' },
        [payment_required.to_json]
      ]
    end
  end

  private

  def should_exclude?(path, config)
    config.exclude_paths.any? { |p| path.start_with?(p) }
  end

  def valid_token?(token)
    return false if token.nil? || token.empty?

    begin
      decoded = Base64.decode64(token)
      decoded.include?('"signature"') && decoded.include?('"sender"')
    rescue
      false
    end
  end

  def create_payment_required(config, path)
    now = Time.now.to_i
    expiry = now + config.expiry_seconds
    salt = Digest::MD5.hexdigest("#{path}:#{now}")[0..15]

    {
      version: 1,
      scheme: config.scheme,
      network: config.network,
      payment_token: config.payment_token,
      max_amount: config.price_per_request.to_s,
      recipient: config.recipient,
      description: config.description,
      expiry: expiry,
      salt: salt,
      instructions: [
        {
          protocol: 'https',
          method: 'POST',
          url: "#{config.facilitator_url}/v1/payment",
          headers: { 'Content-Type' => 'application/json' },
          body: {
            network: config.network,
            payment_token: config.payment_token,
            amount: config.price_per_request.to_s,
            recipient: config.recipient,
            valid_after: now,
            salt: salt
          }
        }
      ]
    }
  end
end
