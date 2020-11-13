pub fn main() {
    
    let secret = "".to_string();
    let params = turbineitor::Params { contest_number : 1, site_number: 1, secret };
    
    let c = turbineitor::establish_connection();
    println!("testing: {:?}", turbineitor::helpers::check_password("admin", "boca", &c, &params));
    
}