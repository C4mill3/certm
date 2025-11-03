use openssl::rsa::{Rsa, Padding};
use openssl::pkey::{PKey, Private, Public};
use openssl::x509::{X509Builder, X509Extension, X509NameBuilder, X509};
use openssl::asn1::{Asn1Time, Asn1Object, Asn1OctetString};
use openssl::sha::sha1;
use std::net::IpAddr;
use std::str;
use rand::Rng;

#[derive(Debug)]
pub enum CertType{
    Server,
    Client,
    ServerAndClient,
    Ca,
    Unknown
}

pub enum KeySize{
    Size1024,
    Size2048,
    Size4096
}

#[derive(Debug)]
pub struct Cert {
    pub cert_type: CertType,
    pub cert: String,
    pub private_key: String,
}

impl Cert {
    pub fn new(ca: &Cert, cert_type: CertType, serial_number: u32, key_size: KeySize, common_name: &str, altname_dns : &Vec<String>, altname_ip: &Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        return generate_new_cert(ca, cert_type, serial_number, key_size, common_name, altname_dns, altname_ip);
    }

    pub fn get_cert(&self) -> Result<X509, Box<dyn std::error::Error >>{
        return Ok(X509::from_pem(self.cert.as_bytes())?);
    }
    pub fn get_private_key(&self) -> Result<PKey<Private>, Box<dyn std::error::Error >>{
        return Ok(PKey::private_key_from_pem(&self.private_key.as_bytes())?);
    }
    pub fn get_public_key(&self) -> Result<PKey<Public>, Box<dyn std::error::Error >>{
        let cert = X509::from_pem(self.cert.as_bytes())?;
        return Ok(cert.public_key()?);
    }

    // Export the public key in PEM format
    pub fn get_cert_txt(&self) -> Result<String, Box<dyn std::error::Error>> {
        let public_key = self.get_public_key()?;
        let public_key_pem = public_key.public_key_to_pem()?;
        Ok(String::from_utf8(public_key_pem)?)
    }

    // Export the private key in PEM format
    pub fn get_private_txt(&self) -> Result<String, Box<dyn std::error::Error>> {
        let private_key = self.get_private_key()?;
        let private_key_pem = private_key.private_key_to_pem_pkcs8()?;
        Ok(String::from_utf8(private_key_pem)?)
    }
    
    // Utility
    pub fn get_info_txt(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cert = X509::from_pem(self.cert.as_bytes())?;
        return Ok(String::from_utf8(cert.to_text()?)?);
    }
    
    pub fn get_subject_name(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cert = X509::from_pem(self.cert.as_bytes())?;
        let subject_name: &openssl::x509::X509NameRef = cert.subject_name();
        for entry in subject_name.entries() {
            let key = entry.object().nid().short_name().unwrap_or_default();
            if key == "CN" {
                // Attempt to convert the data to UTF-8 and return it as a Result
                return entry.data().as_utf8()
                    .map(|value| value.to_string())
                    .map_err(|_| "Failed to convert entry data to UTF-8".into());
            }
        }
        Err("CN not found".into())
    }

    pub fn is_signed_by(&self, ca: &Cert) -> Result<bool, Box<dyn std::error::Error>>{
        let cert = X509::from_pem(self.cert.as_bytes())?;
        let ca_pubkey = ca.get_public_key()?;
        return Ok(cert.verify(&ca_pubkey)?);
    }

}




#[derive(Debug)]
pub struct Realm {
    pub name: String,
    pub ca: Cert,
    pub last_serial_num: u32,
    pub certs: Vec<Cert>,
}

impl Realm {
    pub fn new(name: &'static str, ca_key_size: KeySize, ca_common_name: &str, ca_organization: &str, ca_country: &str) -> Result<Self, Box<dyn std::error::Error>>{
        return new_realm(name, ca_key_size, ca_common_name, ca_organization, ca_country);
    }

    pub fn new_from_ca(name: &'static str, ca_cert: String, ca_private_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse the CA certificate
        let ca_x509 = X509::from_pem(ca_cert.as_bytes()).map_err(|e| e.to_string())?;

        
        // Check if the CA attribute is true
        let cafalse_extension: Vec<u8> = create_extension(&"2.5.29.19", &[0x30, 0x06, 0x01, 0x01, 0xFF, 0x02, 0x01, 0x00], true)?.to_der()?; // X509v3 Basic Constraints (critical): CA:True, pathlen:0
        let ca_der: Vec<u8> = ca_x509.to_der()?;
        if cafalse_extension.len() > ca_der.len() {
            return Err(Box::from("To little to even be a certificate"));
        }

        // Iterate through cert and check for a match
        let mut found = false;
        for i in 0..=ca_der.len() - cafalse_extension.len() {
            if &ca_der[i..i + cafalse_extension.len()] == cafalse_extension {
                found = true;
                break;
            }
        }
        if ! found{
            return Err(Box::from("Cert is missing the Basic Constaint CA:True"));
        }

        // Verify if the private key matches the public key from the certificate
        let public_key_pem = String::from_utf8(ca_x509.public_key()?.public_key_to_pem()?)?;
        

        if ! rsa_keys_match(ca_private_key.as_bytes(), public_key_pem.as_bytes())? {
            return Err(Box::from("The private key does not match the public key in the certificate"));
        }

        let ca: Cert = Cert {
            cert_type: CertType::Ca,
            cert: ca_cert,
            private_key: ca_private_key,
        };

        let certs: Vec<Cert> = Vec::new();
        Ok(Self{
            name: name.to_string(),
            ca,
            last_serial_num: 1000,
            certs,
        })
    }


    pub fn add_cert(&mut self, cert_type: CertType, key_size: KeySize, common_name: &str,  altname_dns : &Vec<String>, altname_ip: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let new_serial_number = self.last_serial_num + 1;
        let new_cert = Cert::new(&self.ca, cert_type, new_serial_number,  key_size, common_name,  altname_dns, altname_ip)?;
        self.certs.push(new_cert);
        self.last_serial_num = new_serial_number;
        Ok(())
    }

    //pub fn add_cert_using_csr(csr : String) -> Result<(), Box<dyn std::error::Error>> {
    //Not implemented yet
    //    Ok(())
    //}

    pub fn import_cert(&mut self, cert: String, private_key: String) -> Result<(), Box<dyn std::error::Error>> {
        // add a cert to a realm, only cert is mandatory,
        // private_key can be empty
        // but if not empty, have to be the matching private

        // Parse the CA certificate
        let cert_x509 = X509::from_pem(cert.as_bytes()).map_err(|e| e.to_string())?;

        fn is_extension_in_cert(cert_der : &Vec<u8>, extension: &Vec<u8>) -> bool {
            if extension.len() > cert_der.len() {
                return false; // previously Err(Box::from("To little to even be a certificate"));
            }

            // Iterate through big_tab and check for a match
            let mut found_extension = false;
            for i in 0..=cert_der.len() - extension.len() {
                if &cert_der[i..i + extension.len()] == extension {
                    found_extension = true;
                }
            }
            return  found_extension;
        }
        // Check if the CA attribute is false
        let cafalse_extension: Vec<u8> = create_extension(&"2.5.29.19", &[0x30, 0x00], true)?.to_der()?; // X509v3 Basic Constraints (critical): CA:False
        let ca_der: Vec<u8> = cert_x509.to_der()?;
        
        if ! is_extension_in_cert(&ca_der, &cafalse_extension){
            return Err(Box::from("Cert is missing the Basic Constaint CA:False"));
        }
        
        let extended_server_extension = create_extension(&"2.5.29.37",&[0x30, 0x0A, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01], false)?.to_der()?; // TLS Web Server Authentication 
        let extended_client_extension = create_extension(&"2.5.29.37",&[0x30, 0x0A, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02], false)?.to_der()?; // TLS Web Client Authentication
        let extended_server_client_extension = create_extension(&"2.5.29.37",&[0x30, 0x14, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02], false)?.to_der()?; // Both TLS Web Server/Client Authentication 
        let extended_client_server_extension = create_extension(&"2.5.29.37",&[0x30, 0x14, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01], false)?.to_der()?; // Both TLS Web Client/Server Authentication 
        
        let cert_type: CertType = {
            if is_extension_in_cert(&ca_der, &extended_server_extension){
                CertType::Server
            }else if is_extension_in_cert(&ca_der, &extended_client_extension) {
                CertType::Client
            }else if is_extension_in_cert(&ca_der, &extended_client_server_extension) ||
                    is_extension_in_cert(&ca_der, &extended_server_client_extension) {
                CertType::ServerAndClient
            }
            else {
                CertType::Unknown
            }
        };

        if ! private_key.is_empty(){ // check or skip if empty
            // Verify if the private key matches the public key from the certificate
            let public_key_pem = String::from_utf8(cert_x509.public_key()?.public_key_to_pem()?)?;
            
            if ! rsa_keys_match(private_key.as_bytes(), public_key_pem.as_bytes())? {
                return Err(Box::from("The private key does not match the public key in the certificate"));
            }
        }

        let new_cert: Cert = Cert {
            cert_type,
            cert,
            private_key
        };
        self.certs.push(new_cert);
        
        // TODO what about last_serial_number update ?
        Ok(())
    }

    pub fn remove_cert(&mut self, cert_id : usize) -> Result<(), Box<dyn std::error::Error>> {
        // Be careful will only remove the cert from the list, it will still be usable until expiration
        if cert_id >= self.certs.len(){
            return Err(Box::from("Cert Id out of bound"))
        }
        self.certs.remove(cert_id);
        Ok(())
    }
}

fn new_realm(name: &'static str, ca_key_size: KeySize, ca_common_name: &str, ca_organization: &str, ca_country: &str) -> Result<Realm, Box<dyn std::error::Error>> {
    let ca = generate_new_ca(ca_key_size, ca_common_name, ca_organization, ca_country)?;
    let certs: Vec<Cert> = Vec::new();
    let resp = Realm{
        name: name.to_string(),
        ca,
        last_serial_num: 1000,
        certs,
    };
    return Ok(resp);
}

fn generate_new_ca(key_size : KeySize, common_name : &str, organization : &str, country : &str) -> Result<Cert, Box<dyn std::error::Error>>{
    //  generate a new CA

    // Generate CA private +pub key
    // Generate private key
    let rsa_size = match key_size {
        KeySize::Size1024 => {1024},
        KeySize::Size2048 => {2048},
        KeySize::Size4096 => {4096}
    };
    let ca_key = Rsa::generate(rsa_size)?;
    let ca_private_key = PKey::from_rsa(ca_key)?;
    let public_key_bytes = ca_private_key.public_key_to_pem()?;

    // Create self-signed CA certificate
    let mut name = X509NameBuilder::new()?;
    name.append_entry_by_text("C", country)?;
    name.append_entry_by_text("O", organization)?;
    name.append_entry_by_text("CN", common_name)?;
    let name = name.build();

    let mut ca_cert_builder = X509Builder::new()?;
    ca_cert_builder.set_version(2)?; // Version 3
    
    let mut rng = rand::rng();
    let random_number: u32 = rng.random::<u32>();
    ca_cert_builder.set_serial_number(openssl::bn::BigNum::from_u32(random_number)?.to_asn1_integer()?.as_ref())?;
    ca_cert_builder.set_not_before(Asn1Time::days_from_now(0)?.as_ref())?;

    ca_cert_builder.set_not_after(Asn1Time::days_from_now(3650)?.as_ref())?;
    ca_cert_builder.set_subject_name(&name)?;
    ca_cert_builder.set_pubkey(&ca_private_key)?;
    
    let hash = sha1(&public_key_bytes);
    // adding hash of pub key in extensions
    let subject_key_identifier_der = {
        let mut der = Vec::new();
        der.push(0x04); // OCTET STRING
        der.push(hash.len() as u8); // Length of the hash
        der.extend_from_slice(&hash); // Add the hash
        der
    };

    let authority_key_identifier_der = {
        let mut der = Vec::new();
        der.extend_from_slice(&[0x30, 0x16, 0x80]); // OCTET STRING
        der.push(hash.len() as u8); // Length of the hash
        der.extend_from_slice(&hash); // Add the authority (myself) hash
        der
    };

    ca_cert_builder.append_extension(create_extension(&"2.5.29.14", &subject_key_identifier_der, false)?)?; // X509v3 Subject Key Identifier: hash
    ca_cert_builder.append_extension(create_extension(&"2.5.29.35",&authority_key_identifier_der, false)?)?; // X509v3 Authority Key Identifier: keyid:always
    
    ca_cert_builder.append_extension(create_extension(&"2.5.29.19", &[0x30, 0x06, 0x01, 0x01, 0xFF, 0x02, 0x01, 0x00], true)?)?; // X509v3 Basic Constraints: CA:TRUE, pathlen:0
    ca_cert_builder.append_extension(create_extension(&"2.5.29.15", &[0x03, 0x02, 0x01, 0x86], true)?)?; // X509v3 Key Usage: Digital Signature, Certificate Sign, CRL Sign
    
    ca_cert_builder.append_extension(create_extension(&"2.5.29.37", &[0x30, 0x14, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02], false)?)?; // extendedKeyUsage
    
    ca_cert_builder.sign(&ca_private_key, openssl::hash::MessageDigest::sha256())?;


    let ca_cert = ca_cert_builder.build();

    // Save CA private key and certificate to struct
    let  ca_cert_text: &Vec<u8> = &ca_cert.to_pem()?;
    let ca_private_text : &Vec<u8>  = &ca_private_key.private_key_to_pem_pkcs8()?;


    let result = Cert {
        cert_type: CertType::Ca,
        cert: str::from_utf8(ca_cert_text)?.to_string(),
        private_key: str::from_utf8(ca_private_text)?.to_string(),
    };
    return Ok(result);

}

fn generate_new_cert(ca: &Cert, cert_type: CertType, serial_number: u32, key_size: KeySize, common_name: &str, altname_dns : &Vec<String>, altname_ip: &Vec<String>) -> Result<Cert, Box<dyn std::error::Error>> {
    // generate a new certificate

    if let CertType::Ca = cert_type {
        return Err(Box::from("Will not generate a sub-ca, not implemented yet"));
    }

    if let CertType::Unknown = cert_type {
        return Err(Box::from("What do you mean, you wants an unknown Cert ?"));
    }

    match ca.cert_type {
        CertType::Ca => {
            // pass  
        }
        _ => {
            return Err(Box::from("Ca cert is not a ca ???"));
        }
    }

    let ca_cert = ca.get_cert()?;
    let ca_private_key = ca.get_private_key()?;
    let ca_public_key_bytes = ca_private_key.public_key_to_pem()?;

    // Generate private key
    let rsa_size = match key_size {
        KeySize::Size1024 => {1024},
        KeySize::Size2048 => {2048},
        KeySize::Size4096 => {4096}
    };
    let key = Rsa::generate(rsa_size)?;
    let private_key = PKey::from_rsa(key)?;
    let public_key_bytes = private_key.public_key_to_pem()?;

    // Create CSR for the new cert
    let mut name = X509NameBuilder::new()?;
    name.append_entry_by_text("CN", common_name)?;
    let name = name.build();

    let mut csr_builder = openssl::x509::X509ReqBuilder::new()?;
    csr_builder.set_subject_name(&name)?;
    csr_builder.set_pubkey(&private_key)?;
    csr_builder.sign(&private_key, openssl::hash::MessageDigest::sha256())?;
    let csr = csr_builder.build();

    // Sign the cert with the CA
    let mut cert_builder = X509Builder::new()?;
    cert_builder.set_version(3)?; // Version 3
    cert_builder.set_serial_number(openssl::bn::BigNum::from_u32(serial_number)?.to_asn1_integer()?.as_ref())?;
    cert_builder.set_not_before(Asn1Time::days_from_now(0)?.as_ref())?;
    cert_builder.set_not_after(Asn1Time::days_from_now(365)?.as_ref())?;
    cert_builder.set_subject_name(csr.subject_name())?;
    cert_builder.set_issuer_name(&ca_cert.subject_name())?;
    cert_builder.set_pubkey(csr.public_key()?.as_ref())?;

    // adding hash of pub key in extensions
    let hash = sha1(&public_key_bytes);
    let subject_key_identifier_der = {
        let mut der = Vec::new();
        der.push(0x04); // OCTET STRING
        der.push(hash.len() as u8); // Length of the hash
        der.extend_from_slice(&hash); // Add the hash
        der
    };
    
    let hash = sha1(&ca_public_key_bytes);
    let authority_key_identifier_der = {
        let mut der = Vec::new();
        der.extend_from_slice(&[0x30, 0x16, 0x80]); // OCTET STRING
        der.push(hash.len() as u8); // Length of the hash
        der.extend_from_slice(&hash); // Add the authority hash
        der
    };

    //Extensions
    cert_builder.append_extension(create_extension(&"2.5.29.14", &subject_key_identifier_der, false)?)?; // X509v3 Subject Key Identifier: hash
    cert_builder.append_extension(create_extension(&"2.5.29.35",&authority_key_identifier_der, false)?)?; // X509v3 Authority Key Identifier: keyid:always

    cert_builder.append_extension(create_extension(&"2.5.29.19", &[0x30, 0x00], true)?)?; // X509v3 Basic Constraints: CA:False
    cert_builder.append_extension(create_extension(&"2.5.29.15", &[0x03, 0x02, 0x05, 0xA0], true)?)?; // X509v3 Key Usage: Digital Signature, Key Encipherment
    
    
    // X509v3 Subject Alternative Name
    if ! (altname_dns.is_empty() && altname_ip.is_empty()){
        cert_builder.append_extension(create_altname_extension(altname_dns, altname_ip)?)?;
    }

    // X509v3 Extended Key Usage
    let ext_key_usage = {
        let mut der = Vec::new();
        der.extend_from_slice(&[0x30, 0x0A, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03]);
        match cert_type {
            CertType::Client => {
                der.push(0x02); // TLS Web Client Authentication
            }
            CertType::Server => {
                der.push(0x01); // TLS Web Server Authentication 
            }
            CertType::ServerAndClient => { // Both TLS Web Server/Client Authentication 
                der[1] = 0x14; // change total size to count both 
                der.push(0x01); // TLS Web Server Authentication 
                der.extend_from_slice(&[0x06, 0x08, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02]); // TLS Web Client Authentication
            }
            CertType::Ca | CertType::Unknown => {
                return Err(Box::from("Will not this type of Cert"));
            }
        }
        der
    };
    cert_builder.append_extension(create_extension(&"2.5.29.37", &ext_key_usage, false)?)?; // extendedKeyUsage 1 = TLS Web Server Authentication 2 = TLS Web Client Authentication

    
    cert_builder.sign(&ca_private_key, openssl::hash::MessageDigest::sha256())?;
    let cert = cert_builder.build();


    
    // Save Cert private key and certificate to struct
    let cert_text: &Vec<u8> = &cert.to_pem()?;
    let private_text : &Vec<u8>  = &private_key.private_key_to_pem_pkcs8()?;

    let result = Cert {
        cert_type: cert_type,
        cert: str::from_utf8(cert_text)?.to_string(),
        private_key: str::from_utf8(private_text)?.to_string(),
    };
    return Ok(result);
}

fn create_extension(oid : &str,der :&[u8], critical : bool) -> Result<X509Extension, Box<dyn std::error::Error>> {
    // All X509 Extensions: https://oid-base.com/get/2.5.29
    // get Asn1 from existing cert: openssl asn1parse -in ca.crt -inform PEM
    
    // Step 1: Create the OID for the extension
    let oid_obj = Asn1Object::from_str(oid)?;
    
    // Step 2: Create the DER contents (this is just an example)
    let der_obj = Asn1OctetString::new_from_bytes(der)?;
    // Populate der_contents as needed
    
    let extension = X509Extension::new_from_der(&oid_obj, critical, &der_obj)?;
    
    Ok(extension)
}

fn size_to_asn1_bytes(size: usize) -> Vec<u8> {
    //allow to express any size to asn1 format (auto switch from short to long format)
    if size <= 127 {
        // Short form: 1 byte for the length
        vec![size as u8]
    } else {
        // Long form: first byte is 0x80 + number of bytes needed to represent the length
        let length_bytes = (size as usize).to_be_bytes();
        let byte_count = length_bytes.iter().take_while(|&&b| b == 0).count();
        let length_of_length = length_bytes.len() - byte_count;

        // Start with the long form identifier
        let mut result = vec![0x80 | length_of_length as u8]; // 0x80 + length of the length

        // Add the actual size bytes
        result.extend_from_slice(&length_bytes[byte_count..]);

        result
    }
}

fn string_to_ip_bits(ip_str: &str) -> Result<Vec<u8>, String> {
    let ip: IpAddr = ip_str.parse().map_err(|_| format!("Invalid IP address: {}", ip_str))?;
    
    match ip {
        IpAddr::V4(addr) => {
            // Convert IPv4 to u8 array
            let octets = addr.octets();
            Ok(octets.iter().map(|&octet| octet).collect())
        },
        IpAddr::V6(addr) => {
            // Convert IPv6 to u8 array
            let segments = addr.segments();
            let mut hex_bytes = Vec::new();
            for segment in segments {
                hex_bytes.push((segment >> 8) as u8);  // High byte
                hex_bytes.push((segment & 0xFF) as u8); // Low byte
            }
            Ok(hex_bytes)
        },
    }
}

fn create_altname_extension(dns: &[String], ip: &[String]) -> Result<X509Extension, Box<dyn std::error::Error>> {
    // generate the X509v3 Subject Alternative Name extensions
    // always used because DNS.1 is mandatory, optionnaly add other DNS.x and IP.x 
    // https://www.rfc-editor.org/rfc/rfc5280.html#section-4.2.1.6
    
    if dns.is_empty() && ip.is_empty() {
        return Err(Box::from("Error: Both DNS and IP lists are empty."));
    }

    let mut contents: Vec<u8> = Vec::new();

    for element in dns{
        let length: usize= element.len();
        if length != 0{
            let bytes_vec: Vec<u8> = element.clone().into_bytes();
            contents.push(0x82);
            contents.extend_from_slice(&size_to_asn1_bytes(length));
            contents.extend_from_slice(&bytes_vec);
        }
    }

    for element in ip{
        let ip_formated = string_to_ip_bits(&element)?;
        let length: usize= ip_formated.len();
        if length != 0{
            contents.push(0x87);
            contents.extend_from_slice(&size_to_asn1_bytes(length));
            contents.extend_from_slice(&ip_formated);
        }
    }
    

    let der_final = { // create final der
        let mut der = Vec::new();
        der.push(0x30); // SEQUENCE type
        der.extend_from_slice(&size_to_asn1_bytes(contents.len())); // Total Length
        der.extend_from_slice(&contents); // Add the content
        der
    };

    Ok(create_extension("2.5.29.17", &der_final, false)?)
}

fn rsa_keys_match(private_key_pem: &[u8], public_key_pem: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    // check if a private / public key association match
    // Load the private key
    let private_key = Rsa::private_key_from_pem(private_key_pem)?;
    
    // Load the public key
    let public_key = Rsa::public_key_from_pem(public_key_pem)?;
    
    // Encrypt a test message with the private key
    let test_message = b"test";
    let mut encrypted = vec![0; private_key.size() as usize];
    let encrypted_len = private_key.private_encrypt(test_message, &mut encrypted, Padding::PKCS1)?;
    
    // Decrypt the message with the public key
    let mut decrypted = vec![0; public_key.size() as usize];
    let decrypted_len = public_key.public_decrypt(&encrypted[..encrypted_len], &mut decrypted, Padding::PKCS1)?;
    
    // Check if the decrypted message matches the original
    Ok(&decrypted[..decrypted_len] == test_message)
}

//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    
//
//    let mut realm= Realm::new("test", KeySize::Size1024, "test.com", "TEST", "FR")?;
//    realm.add_cert(CertType::ServerAndClient, KeySize::Size1024, "home.org", &vec![String::from("home.org")], &vec![])?;
//    realm.add_cert(CertType::Server, KeySize::Size1024, "POLS", &vec![String::from("TEST")], &vec![])?;
//
//    println!("{:?}", realm);
//
//    let cert1= &realm.certs[0];
//    println!("See this:\n{}", cert1.get_cert_txt()?);
//    println!("This is private:\n{}", cert1.get_private_txt()?);
//    println!("here is the truth: {}", cert1.is_signed_by(&realm.ca)?);
//    println!("here is what you shouldn't see: {}", cert1.is_signed_by(&cert1)?);
//    println!("{}", &realm.certs[0].get_info_txt()?);
//    println!("{}", &realm.certs[0].get_subject_name()?);
//    
//    
//    Ok(())
//}