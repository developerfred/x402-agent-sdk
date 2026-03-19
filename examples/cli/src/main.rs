use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "x402")]
#[command(about = "x402 CLI - Payment-enabled API toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        template: Option<String>,
    },
    Serve {
        port: Option<u16>,
        network: Option<String>,
    },
    Pay {
        amount: Option<String>,
        recipient: Option<String>,
    },
    CreateSession {
        max_amount: Option<String>,
        duration: Option<u64>,
    },
    Verify {
        tx_hash: Option<String>,
    },
    Config,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { template } => {
            println!(
                "✓ Initializing {} with x402...",
                template.as_deref().unwrap_or("express")
            );
        }
        Commands::Serve { port, network } => {
            println!("🚀 Starting x402 server...");
            println!("   Port: {}", port.unwrap_or(3000));
            println!("   Network: {}", network.as_deref().unwrap_or("eip155:1"));
        }
        Commands::Pay { amount, recipient } => {
            println!("💳 Processing payment...");
            println!("   Amount: {:?}", amount);
            println!("   Recipient: {:?}", recipient);
        }
        Commands::CreateSession {
            max_amount,
            duration,
        } => {
            println!("🔐 Creating MPP session...");
            println!("   Max amount: {:?}", max_amount);
            println!("   Duration: {}s", duration.unwrap_or(3600));
        }
        Commands::Verify { tx_hash } => {
            println!("✅ Verifying payment...");
            println!("   Tx: {:?}", tx_hash);
        }
        Commands::Config => {
            println!("⚙️  x402 Configuration");
        }
    }
}
