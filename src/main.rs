mod tools;


fn main(){
    tools::certs_manager::generate_new_ca(4096, "aa", "ESIEA", "FR", "Paris", "jsp");
    let a = tools::certs_manager::list_ca();
    println!("res: {:?}", a);
    return;
} 
