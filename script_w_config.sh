#!/bin/bash
# Var
CA_DIR="myCA"
KEY_DIR="$CA_DIR/private"
CERT_DIR="$CA_DIR/certs" 
NEWCERTS_DIR="$CA_DIR/newcerts"
INDEX_FILE="$CA_DIR/index.txt"
SERIAL_FILE="$CA_DIR/serial"
CA_KEY="$KEY_DIR/ca.key"
CA_CERT="$CERT_DIR/ca.crt"
SERVER_KEY="$KEY_DIR/server.key"
SERVER_CSR="$CERT_DIR/server.csr"
SERVER_CERT="$CERT_DIR/server.crt" 

# Créer les répertoires nécessaires
mkdir -p $KEY_DIR $CERT_DIR $NEWCERTS_DIR
touch $INDEX_FILE
echo 1000 > $SERIAL_FILE  # Générer la clé privée du CA # pas compris TODO
openssl genpkey -algorithm RSA -out $CA_KEY -pkeyopt rsa_keygen_bits:2048
# Créer le certificat auto-signé de la CA
openssl req -key $CA_KEY -new -x509 -out $CA_CERT -days 3650 -subj "/C=FR/ST=State/L=City/O=Organization/CN=My CA"
# Générer la clé privée pour le serveur
openssl genpkey -algorithm RSA -out $SERVER_KEY -pkeyopt rsa_keygen_bits:2048
# Créer une demande de signature de certificat (CSR Certificat Signature Request) pour le serveur
openssl req -new -key $SERVER_KEY -out $SERVER_CSR -subj "/C=FR/ST=State/L=City/O=Organization/CN=www.example.com"
# Signer le certificat du serveur avec la CA
openssl x509 -req -in $SERVER_CSR -CA $CA_CERT -CAkey $CA_KEY -CAcreateserial -out $SERVER_CERT -days 365 -extfile <(printf "subjectAltName=DNS:www.example.com")

echo "Autorité de certification et certificat du serveur créés avec succès." 
