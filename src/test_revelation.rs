use data::configdata::ConfigContest;
use data::revelation::RevelationDriver;
use server::load_data_from_url_maybe;

pub async fn build_revelation(input_file: &str) -> Vec<String> {
    let (_, contest_data, runs_data) = load_data_from_url_maybe(input_file.to_string())
        .await
        .expect("Should have loaded file");

    let sedes = ConfigContest::dummy();

    let mut driver = RevelationDriver::new(contest_data, runs_data, sedes);
    let mut result = Vec::new();

    while driver.len() > 0 {
        result.push(format!("{}, {}", driver.peek().unwrap(), driver.len()));
        driver.reveal_step();
    }
    return result;
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    use tokio;

    async fn check_revelation(input_file: &str, golden_model: &str) {
        let model = read_lines(golden_model).expect("Should be able to read golden model");
        let reveals = super::build_revelation(input_file).await;

        for (expected, resulted) in model.into_iter().zip(reveals.into_iter()) {
            let expected_string = expected.expect("Should be able to read line from golden model");
            assert_eq!(expected_string, resulted);
        }
    }

    #[tokio::test]
    async fn test() {
        check_revelation(
            "tests/inputs/1a_fase_2021_frozen_unlocked_argentina.zip",
            "tests/inputs/1a_fase_2021_frozen_unlocked_argentina.zip.revelation",
        )
        .await;
    }
}
