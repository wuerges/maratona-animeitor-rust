use data::revelation::RevelationDriver;
use service::webcast::load_data_from_url_maybe;

pub async fn build_revelation(input_file: &str) -> color_eyre::eyre::Result<Vec<String>> {
    let (_, contest_data, runs_data) = load_data_from_url_maybe(input_file).await?;

    let mut driver = RevelationDriver::new(contest_data, runs_data);
    let mut result = Vec::new();

    while !driver.is_empty() {
        result.push(format!("{}, {}", driver.peek().unwrap(), driver.len()));
        driver.reveal_step();
    }
    Ok(result)
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

    use rstest::rstest;
    use tokio;

    async fn check_revelation(input_file: &str, golden_model: &str) {
        let model = read_lines(golden_model).expect("Should be able to read golden model");
        let reveals = super::build_revelation(input_file).await.unwrap();

        for (expected, resulted) in model.into_iter().zip(reveals.into_iter()) {
            let expected_string = expected.expect("Should be able to read line from golden model");
            assert_eq!(expected_string, resulted);
        }
    }

    #[rstest]
    #[case("../tests/inputs/webcast_jones_2021_judge_submission.zip")]
    #[case("../tests/inputs/webcast_jones_08_2021.zip")]
    #[case("../tests/inputs/webcast_jones_2021.zip")]
    #[case("../tests/inputs/webcast_jones.zip")]
    #[case("../tests/inputs/1a_fase_2021_frozen_unlocked_argentina.zip")]
    #[tokio::test]
    async fn test_golden_model_fast_tests(#[case] test_input: &str) {
        check_revelation(test_input, &format!("{test_input}.revelation")).await;
    }

    #[cfg(feature = "slow_tests")]
    #[rstest]
    #[case("../tests/inputs/webcast_early_frozen.zip")]
    #[case("../tests/inputs/webcast_frozen_aquecimento_1a_fase_2021_frozen_unlocked.zip")]
    #[case("../tests/inputs/warmup_2a_fase_2020_fake_frozen.zip")]
    #[case("../tests/inputs/webcast_frozen_aquecimento_1a_fase_2021.zip")]
    #[case("../tests/inputs/webcast_frozen_aquecimento_1a_fase_2020.zip")]
    #[case("../tests/inputs/1a_fase_2021_frozen_unlocked.zip")]
    #[case("../tests/inputs/webcast_1573336220.zip")]
    #[case("../tests/inputs/webcast_zip_1a_fase_2020.zip")]
    #[tokio::test]
    async fn test_golden_model_slow_tests(#[case] test_input: &str) {
        check_revelation(test_input, &format!("{test_input}.revelation")).await;
    }
}
