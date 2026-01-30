# ⚡ LNURL Project - Lightning Network Protocol Implementation

Complete implementation of LNURL protocols for Lightning Network:
- **LUD-02**: Channel Request
- **LUD-03**: Withdraw Request  
- **LUD-04**: LNURL-auth (authentication)

## 📋 Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Bitcoin Core](https://bitcoin.org/en/download) in testnet4 mode
- [Core Lightning](https://github.com/ElementsProject/lightning) version 25.12.1+

## 🚀 Local Installation

## 🚀 Local Installation

### 1. Setup Bitcoin Core (testnet4)

```bash
# Start Bitcoin Core in testnet4 mode
bitcoind -testnet4 -daemon

# Verify it's running
bitcoin-cli -testnet4 getblockchaininfo
```

### 2. Setup Core Lightning

```bash
# Create config file
mkdir -p ~/.lightning
cat > ~/.lightning/config << EOF
network=testnet4
log-level=debug
EOF

# Start Core Lightning
lightningd --network=testnet4 --daemon

# Create a wallet and get an address
lightning-cli --network=testnet4 newaddr

# Get testnet4 coins from a faucet
# Faucet: https://mempool.space/testnet4/faucet
# Or send to: tb1q0dzcgv7scppjxsnwlzpkt02vlmc5rtr40wyjgr

# Check your balance
lightning-cli --network=testnet4 listfunds
```

### 3. Server Configuration

**IMPORTANT**: Before running the server, modify the constants in `src/server.rs`:

```rust
const PUBLIC_KEY: &str = "YOUR_NODE_PUBKEY"; // Get it with: lightning-cli getinfo | grep id
const IP_PORT: &str = "YOUR_IP:9735";        // Your public IP
const SERVER_URL: &str = "http://YOUR_IP:3000"; // Your server URL
```

To get your node pubkey:
```bash
lightning-cli --network=testnet4 getinfo | grep id
```

### 4. Run the Project

```bash
# Terminal 1: Start the server
cargo run --bin server

# Terminal 2: Start the client (tests)
cargo run --bin client
```

## 🌐 VPS Deployment (Optional)

### Recommended VPS Providers

- **Contabo**: ~4€/month, good value
- **Hetzner**: ~5€/month, very stable
- **DigitalOcean**: ~6$/month, easy to use

### Quick VPS Setup

```bash
# 1. Connect to VPS
ssh root@YOUR_IP

# 2. Install dependencies
apt update && apt upgrade -y
apt install -y build-essential curl git

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Install Bitcoin Core
wget https://bitcoincore.org/bin/bitcoin-core-28.0/bitcoin-28.0-x86_64-linux-gnu.tar.gz
tar xzf bitcoin-28.0-x86_64-linux-gnu.tar.gz
sudo install -m 0755 -o root -g root -t /usr/local/bin bitcoin-28.0/bin/*

# 5. Install Core Lightning
git clone https://github.com/ElementsProject/lightning.git
cd lightning
git checkout v25.12.1
./configure
make
sudo make install

# 6. Configure Bitcoin Core
mkdir -p ~/.bitcoin
cat > ~/.bitcoin/bitcoin.conf << EOF
testnet4=1
server=1
daemon=1
txindex=1
EOF

# 7. Start Bitcoin and Lightning
bitcoind -daemon
sleep 10
lightningd --network=testnet4 --daemon

# 8. Clone your project
cd ~
git clone git@github.com:YOUR_USERNAME/lnurl-project.git
cd lnurl-project

# 9. Update src/server.rs with your node info

# 10. Build and run
cargo build --release
./target/release/server
```

### Firewall Configuration

```bash
# Open required ports
ufw allow 9735/tcp  # Lightning P2P
ufw allow 3000/tcp  # LNURL Server
ufw enable
```

### Run as systemd service

```bash
# Create service
sudo tee /etc/systemd/system/lnurl-server.service << EOF
[Unit]
Description=LNURL Server
After=network.target lightningd.service

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/$USER/lnurl-project
ExecStart=/home/$USER/lnurl-project/target/release/server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable lnurl-server
sudo systemctl start lnurl-server

# Check logs
sudo journalctl -u lnurl-server -f
```

## 🧪 Testing

The client provides an interactive menu to test each feature:

```
⚡ LNURL Client - Test Suite
============================

What would you like to test?
1. LUD-02: Channel Request
2. LUD-03: Withdraw Request
3. LUD-04: LNURL-auth
4. Run all tests
0. Exit
```

**Note**: LUD-02 and LUD-03 require two different nodes to work properly (you cannot connect/pay yourself). LUD-04 (auth) can be tested locally.

## 📨 Information for Testing

Once everything works:

1. **Get your node pubkey**:
```bash
lightning-cli --network=testnet4 getinfo | grep id
```

2. **Test the endpoints**:
```bash
# Channel request
curl http://YOUR_IP:3000/channel-request

# Withdraw request
curl http://YOUR_IP:3000/withdraw-request

# Auth challenge
curl http://YOUR_IP:3000/auth-challenge
```

## 🐛 Debugging

### Server logs
```bash
sudo journalctl -u lnurl-server -f
```

### Lightning logs
```bash
tail -f ~/.lightning/testnet4/log
```

### Check Lightning is accessible
```bash
lightning-cli --network=testnet4 getinfo
```

## 📚 Resources

- [LUD-02 spec](https://github.com/lnurl/luds/blob/luds/02.md)
- [LUD-03 spec](https://github.com/lnurl/luds/blob/luds/03.md)
- [LUD-04 spec](https://github.com/lnurl/luds/blob/luds/04.md)
- [Core Lightning docs](https://docs.corelightning.org/)
- [cln-rpc docs](https://docs.rs/cln-rpc/latest/cln_rpc/)

## ✅ Project Status

### Completed
- [x] Bitcoin Core setup on testnet4
- [x] Core Lightning setup and configuration
- [x] Wallet funded from testnet4 faucet
- [x] LUD-02: Channel Request implementation
  - [x] `/channel-request` endpoint
  - [x] `/channel-callback` endpoint
- [x] LUD-03: Withdraw Request implementation
  - [x] `/withdraw-request` endpoint
  - [x] `/withdraw-callback` endpoint
- [x] LUD-04: LNURL-auth implementation
  - [x] `/auth-challenge` endpoint
  - [x] `/auth-response` endpoint
  - [x] Signature verification working
- [x] Interactive client with test menu
- [x] Code tested locally
- [x] Repository pushed to GitHub

### Todo (Optional)
- [ ] VPS deployment for 24/7 availability
- [ ] Systemd service configuration
- [ ] Firewall setup on VPS
- [ ] Testing with professor's node

## 📋 Current Configuration

- **Node Pubkey**: `029249978ef61cf264d2cf57589c96780bdd86266fdc065d6b54c48d2c9ea3ad40`
- **Network**: testnet4
- **Server Port**: 3000
- **Lightning Port**: 9735

## 🎯 Project Information

This project implements three LNURL protocols:

1. **LUD-02 (Channel Request)**: Allows requesting channel opening
2. **LUD-03 (Withdraw Request)**: Allows requesting Lightning withdrawals
3. **LUD-04 (LNURL-auth)**: Cryptographic authentication using Lightning node keys

All three protocols are implemented and working. LUD-04 has been tested successfully. LUD-02 and LUD-03 require two different Lightning nodes to test properly (cannot connect/pay to yourself).

---

**Ready for submission! 🎉**

Good luck! ⚡🚀
