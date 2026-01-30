# ⚡ LNURL Project - Lightning Network Protocol Implementation

Implementation complète des protocoles LNURL pour Lightning Network :
- **LUD-02**: Channel Request (demande d'ouverture de channel)
- **LUD-03**: Withdraw Request (demande de retrait)
- **LUD-04**: LNURL-auth (authentification)

## 📋 Prérequis

- [Rust](https://rustup.rs/) (dernière version stable)
- [Bitcoin Core](https://bitcoin.org/en/download) en mode testnet4
- [Core Lightning](https://github.com/ElementsProject/lightning) version 25.12.1+

## 🚀 Installation locale

### 1. Setup Bitcoin Core (testnet4)

```bash
# Démarrer Bitcoin Core en testnet4
bitcoind -testnet4 -daemon

# Vérifier que ça tourne
bitcoin-cli -testnet4 getblockchaininfo
```

### 2. Setup Core Lightning

```bash
# Créer le fichier de config
mkdir -p ~/.lightning
cat > ~/.lightning/config << EOF
network=testnet4
log-level=debug
EOF

# Démarrer Core Lightning
lightningd --network=testnet4 --daemon

# Créer un wallet et obtenir une adresse
lightning-cli --network=testnet4 newaddr

# Récupérer des coins depuis un faucet testnet4
# Faucet: https://mempool.space/testnet4/faucet
# Ou: tb1q0dzcgv7scppjxsnwlzpkt02vlmc5rtr40wyjgr

# Vérifier les funds
lightning-cli --network=testnet4 listfunds
```

### 3. Configuration du serveur

**IMPORTANT**: Avant de lancer le serveur, modifie les constantes dans `src/server.rs`:

```rust
const PUBLIC_KEY: &str = "TON_NODE_PUBKEY"; // Remplace par le résultat de `lightning-cli getinfo | jq -r .id`
const IP_PORT: &str = "TON_IP:9735";        // Remplace par ton IP publique
const SERVER_URL: &str = "http://TON_IP:3000"; // URL de ton serveur
```

Pour obtenir ton node pubkey:
```bash
lightning-cli --network=testnet4 getinfo | grep id
```

### 4. Lancer le projet

```bash
# Terminal 1: Lancer le serveur
cargo run --bin server

# Terminal 2: Lancer le client (tests)
cargo run --bin client
```

## 🌐 Déploiement sur VPS

### Option 1: VPS recommandés

- **Contabo**: ~4€/mois, bon rapport qualité/prix
- **Hetzner**: ~5€/mois, très stable
- **DigitalOcean**: ~6$/mois, simple d'utilisation

### Option 2: Setup sur VPS

```bash
# 1. Se connecter au VPS
ssh root@TON_IP

# 2. Installer les dépendances
apt update && apt upgrade -y
apt install -y build-essential curl git

# 3. Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Installer Bitcoin Core
wget https://bitcoincore.org/bin/bitcoin-core-28.0/bitcoin-28.0-x86_64-linux-gnu.tar.gz
tar xzf bitcoin-28.0-x86_64-linux-gnu.tar.gz
sudo install -m 0755 -o root -g root -t /usr/local/bin bitcoin-28.0/bin/*

# 5. Installer Core Lightning
git clone https://github.com/ElementsProject/lightning.git
cd lightning
git checkout v25.12.1
./configure
make
sudo make install

# 6. Configurer Bitcoin Core
mkdir -p ~/.bitcoin
cat > ~/.bitcoin/bitcoin.conf << EOF
testnet4=1
server=1
daemon=1
txindex=1
EOF

# 7. Démarrer Bitcoin et Lightning
bitcoind -daemon
sleep 10
lightningd --network=testnet4 --daemon

# 8. Cloner ton projet
cd ~
git clone git@github.com:TON_USERNAME/lnurl-project.git
cd lnurl-project

# 9. Modifier src/server.rs avec tes infos (voir étape 3 plus haut)

# 10. Compiler et lancer
cargo build --release
./target/release/server
```

### Configuration du firewall

```bash
# Ouvrir les ports nécessaires
ufw allow 9735/tcp  # Lightning P2P
ufw allow 3000/tcp  # Serveur LNURL
ufw enable
```

### Lancer en background (systemd)

```bash
# Créer le service
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

# Activer et démarrer
sudo systemctl daemon-reload
sudo systemctl enable lnurl-server
sudo systemctl start lnurl-server

# Vérifier les logs
sudo journalctl -u lnurl-server -f
```

## 📡 Tester avec le serveur du prof

Pour tester ton client avec le serveur du prof, modifie `src/client.rs`:

```rust
const SERVER_URL: &str = "http://IP_DU_PROF:3000";
```

Puis lance:
```bash
cargo run --bin client
```

## 🧪 Tests

Le client propose un menu interactif pour tester chaque fonctionnalité:

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

## 📨 Envoyer au prof

Une fois que tout marche:

1. **Récupère ton node pubkey**:
```bash
lightning-cli --network=testnet4 getinfo | grep id
```

2. **Envoie au prof en DM Discord**:
   - Node pubkey: `02xxx...`
   - Repo client: `https://github.com/TON_USERNAME/lnurl-project`
   - Repo server: `https://github.com/TON_USERNAME/lnurl-project`
   - IP du serveur: `TON_IP:3000`

3. **Test les endpoints**:
```bash
# Channel request
curl http://TON_IP:3000/channel-request

# Withdraw request
curl http://TON_IP:3000/withdraw-request

# Auth challenge
curl http://TON_IP:3000/auth-challenge
```

## 🐛 Debug

### Logs du serveur
```bash
sudo journalctl -u lnurl-server -f
```

### Logs de Lightning
```bash
tail -f ~/.lightning/testnet4/log
```

### Vérifier que Lightning est accessible
```bash
lightning-cli --network=testnet4 getinfo
```

## 📚 Ressources

- [LUD-02 spec](https://github.com/lnurl/luds/blob/luds/02.md)
- [LUD-03 spec](https://github.com/lnurl/luds/blob/luds/03.md)
- [LUD-04 spec](https://github.com/lnurl/luds/blob/luds/04.md)
- [Core Lightning docs](https://docs.corelightning.org/)
- [cln-rpc docs](https://docs.rs/cln-rpc/latest/cln_rpc/)

## 🎯 Checklist finale

- [ ] Bitcoin Core sync sur testnet4
- [ ] Core Lightning fonctionne
- [ ] Wallet Lightning financé (depuis faucet)
- [ ] Serveur LNURL accessible publiquement
- [ ] Tests passent avec le client
- [ ] Infos envoyées au prof en DM
- [ ] Serveur reste en ligne pour les tests du prof

Bon courage ! ⚡🚀
