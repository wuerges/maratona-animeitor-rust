pub fn main() {
    
    let params = turbineitor::Params { contest_number : 1, site_number: 1 };
    
    let c = turbineitor::establish_connection();
    println!("testing: {:?}", turbineitor::helpers::check_password("admin", "boca1", &c, &params));
    
}