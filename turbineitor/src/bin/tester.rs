pub fn main() {
    
    let secret = "".to_string();
    let params = turbineitor::Params::new(1, 1, secret);
    
    println!("testing: {:?}", turbineitor::helpers::check_password("admin", "boca", &params));
    
}