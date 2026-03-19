require 'base64'
require 'json'
require 'digest'
require 'time'

module X402Rails
  class Configuration
    attr_accessor :price_per_request, :scheme, :network, :payment_token, :recipient
    attr_accessor :description, :expiry_seconds, :facilitator_url, :exclude_paths

    def initialize
      @price_per_request = 100
      @scheme = 'v1-eip155-exact'
      @network = 'eip155:1'
      @payment_token = ''
      @recipient = ''
      @description = 'API access'
      @expiry_seconds = 3600
      @facilitator_url = 'https://facilitator.x402.rs'
      @exclude_paths = ['/docs', '/rails/info', '/rails/live', '/static']
    end
  end

  class PaymentRequired
    attr_reader :version, :scheme, :network, :payment_token, :max_amount
    attr_reader :recipient, :description, :expiry, :salt, :instructions

    def initialize(config, path)
      @version = 1
      @scheme = config.scheme
      @network = config.network
      @payment_token = config.payment_token
      @max_amount = config.price_per_request.to_s
      @recipient = config.recipient
      @description = config.description
      @expiry = Time.now.to_i + config.expiry_seconds
      @salt = Digest::MD5.hexdigest("#{path}:#{Time.now.to_i}")[0..15]

      @instructions = [
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
            valid_after: Time.now.to_i,
            salt: @salt
          }
        }
      ]
    end

    def to_h
      {
        version: @version,
        scheme: @scheme,
        network: @network,
        payment_token: @payment_token,
        max_amount: @max_amount,
        recipient: @recipient,
        description: @description,
        expiry: @expiry,
        salt: @salt,
        instructions: @instructions
      }
    end
  end

  class Middleware
    def initialize(app, config = nil)
      @app = app
      @config = config || Configuration.new
    end

    def call(env)
      path = env['PATH_INFO']

      if should_exclude?(path)
        return @app.call(env)
      end

      token_header = env['HTTP_X_PAYMENT_TOKEN']

      if valid_token?(token_header)
        return @app.call(env)
      end

      payment_required = PaymentRequired.new(@config, path)

      [
        402,
        { 'Content-Type' => 'application/json' },
        [payment_required.to_h.to_json]
      ]
    end

    private

    def should_exclude?(path)
      @config.exclude_paths.any? { |p| path.start_with?(p) }
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
  end

  class Railtie < Rails::Railtie
    config.x402 = Configuration.new

    initializer 'x402.configure' do |app|
      app.config.x402 = Configuration.new
    end

    config.app_middleware.use Middleware, ->(config) { config.x402 }
  end
end
