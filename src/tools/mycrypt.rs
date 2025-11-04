use aes::Aes256;
use bincode::config::BigEndian;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::Rng;
use sha2::{Sha256, Digest};
use bincode::{self, config::Configuration};

use super::utility::{sanitize_name, resolve_path, write_to_file, CA_VAULT};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const BINCODE_CONFIG: Configuration<BigEndian> = bincode::config::standard()
    .with_big_endian()
    .with_variable_int_encoding();

#[derive(Debug)] // TODO Needed for printing the struct (for demonstration only)
#[derive(bincode::Encode, bincode::Decode)] //Automatically implement Encode and Decode traits
pub struct MyData {
    pub name: String,
    pub age: u8,
}

pub fn encrypt_to_file (name : &str, password: &str, data : &MyData) -> Result<(), Box<dyn std::error::Error>> {


    // serialize struct to bytes
    let encoded_data = bincode::encode_to_vec(
        data,
        BINCODE_CONFIG,
    ).unwrap();

    // init crypto
    let key = Sha256::digest(password);
    let mut rng = rand::rng();
    let iv = rng.random::<[u8; 16]>(); // Random IV
    let cipher = Aes256Cbc::new_from_slices(&key, &iv).unwrap(); // Cipher "tool" for crypto operations

    // encrypt data and format
    let encrypted_data = cipher.encrypt_vec(&encoded_data);
    let mut combined_data = iv.to_vec(); // Complete by puting random in first block
    combined_data.extend(encrypted_data);

    // Write to file (CA_VAULT + name.dat)
    let path_build = &resolve_path(CA_VAULT)
    .join(format!("{}.dat",sanitize_name(name)));
    write_to_file(&path_build, &combined_data, 0o600)?;
    Ok(())
}

pub fn decrypt_from_file (name : &str, password: &str) -> Result<MyData, Box<dyn std::error::Error>> {

    // read file
    let path_build = &resolve_path(CA_VAULT)
        .join(format!("{}.dat", sanitize_name(name)));

    let file_content = std::fs::read(path_build)?;
    
    // init crypto
    let (extracted_iv, encrypted_data) = file_content.split_at(16); // Get the first 16 bytes as IV
    let key = Sha256::digest(password);
    let cipher = Aes256Cbc::new_from_slices(&key, extracted_iv)?; // Cipher "tool" for crypto operations
    // decrypt and deserialize data
    let decrypted_data = cipher.decrypt_vec(encrypted_data)?;

    let deserialize = bincode::decode_from_slice(&decrypted_data, BINCODE_CONFIG);
    let resp = match deserialize{
        Ok((data, _)) => data,
        Err(e) => {
            return Err(Box::new(e));
        },
    };
    
    return Ok(resp);
}


//fn main(){
//    let password = "my_secret_password";
//    let data = MyData {
//        name: "aaaaa".to_string(),
//        age: 30,
//    };
//    println!("Data original: {:?}",  &data);
//    let _ = encrypt_to_file("mydata", password, &data);
//
//    let decrypted_data = decrypt_from_file("mydata", "my_secret_password");
//    println!("Decrypted data: {:?}",  &decrypted_data);
//}
