mod front;
use front::App;

mod tools;
use tools::utility::{CA_VAULT, resolve_path, create_folder};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    create_folder(&resolve_path(CA_VAULT)?, 0o700)?;

    // create app and run it
    let mut app = App::new();
    let res = app.run();

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// Test mycrypt
//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let password = "my_secret_password";
//    let data = mycrypt::MyData {
//        name: "aaaaa".to_string(),
//        age: 30,
//    };
//    println!("Data original: {:?}",  &data);
//    let _ = mycrypt::encrypt_to_file("mydata", password, &data)?;
//  
//
//    let decrypted_data = mycrypt::decrypt_from_file("mydata", "my_secret_password");
//    println!("Decrypted data: {:?}",  &decrypted_data);
//    Ok(())
//}

// Test Cert_Manager
//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let mut realm= certs_manager::Realm::new("test", certs_manager::KeySize::Size1024, "test.com", "TEST", "FR")?;
//    realm.add_cert(certs_manager::CertType::ServerAndClient, certs_manager::KeySize::Size1024, "home.org", &vec![String::from("home.org")], &vec![])?;
//    realm.add_cert(certs_manager::CertType::Server, certs_manager::KeySize::Size1024, "POLS", &vec![String::from("TEST")], &vec![])?;
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