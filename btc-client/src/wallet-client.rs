// fn main() {
// pub trait WalletClient {
//     async fn onchain_address(&self) -> Result<OnchainAddress, WalletError>;
//     async fn send_onchain_payment(
//         &self,
//         destination: String,
//         amount_in_sats: Decimal,
//         memo: Option<String>,
//         confirmations: usize,
//     ) -> Result<(), WalletError>;
// }

// #[derive(Debug)]
// pub struct OnchainAddress {
//     pub address: String,
// }
// }

// Define a trait for a Bitcoin wallet
pub trait BitcoinWallet {
    // Method for generating a new Bitcoin address
    fn generate_address(&self) -> String;

    // Method for sending Bitcoin to a specified address
    fn send(&self, address: &str, amount: f64) -> bool;

    // Method for checking the current balance of the wallet
    fn check_balance(&self) -> f64;
}

// Implement the BitcoinWallet trait for a struct representing a Bitcoin wallet
pub struct MyBitcoinWallet {
    // Add fields for the wallet's private key, public key, and balance
    private_key: String,
    public_key: String,
    balance: f64,
}

impl BitcoinWallet for MyBitcoinWallet {
    // Implement the generate_address method using the wallet's public key
    fn generate_address(&self) -> String {
        format!("Bitcoin address for public key {}", self.public_key)
    }

    // Implement the send method by subtracting the specified amount from the wallet's balance
    fn send(&self, address: &str, amount: f64) -> bool {
        if amount > self.balance {
            return false;
        }
        self.balance -= amount;
        true
    }

    // Implement the check_balance method to return the wallet's current balance
    fn check_balance(&self) -> f64 {
        self.balance
    }
}

fn main() {
    // Create a new instance of MyBitcoinWallet
    let my_wallet = MyBitcoinWallet {
        private_key: "my_private_key".to_string(),
        public_key: "my_public_key".to_string(),
        balance: 100.0,
    };

    // Use the BitcoinWallet methods to interact with the wallet
    let address = my_wallet.generate_address();
    let balance = my_wallet.check_balance();
    let sent_successfully = my_wallet.send(&address, 50.0);
}