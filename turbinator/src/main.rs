extern crate diesel;

use self::diesel::prelude::*;

use turbinator::models::*;
use turbinator::schema::*;
use turbinator::*;
// use turbinator::models::*;

fn main() {

    use runtable::dsl;

    let connection = establish_connection();

    let runs : i64 = dsl::runtable.count()
        .get_result(&connection)
        .expect("Error counting runs");

    println!("Displaying {:?} runs", runs);
    let results = dsl::runtable
        .limit(5)
        .load::<Runtable>(&connection)
        .expect("Error loading runs");
    for run in results {
        println!("{}", run.usernumber);
        println!("----------\n");
        println!("{}", run.runproblem);
    }
}