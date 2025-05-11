mod tools;


fn main(){
    let b = tools::certs_manager::generate_new_ca(4096, "test", "ESIEA", "FR");
    let a = tools::certs_manager::list_ca();
    println!("res1 {:?}", b);
    println!("res: {:?}", a);
    let c = tools::certs_manager::generate_new_cert("test", 4096, "test.com", "ESIEA", "FR");
    println!("ress : {:?}", c);
    return;
} 
