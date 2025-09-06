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

    async fn check_revelation(
        input_file: &str,
        golden_model: &str,
    ) -> color_eyre::eyre::Result<()> {
        let model = read_lines(golden_model)?.collect::<Result<Vec<_>, _>>()?;
        let reveals = super::build_revelation(input_file).await?;

        assert_eq!(model, reveals, "golden models differ {}", input_file);

        Ok(())
    }

    #[rstest]
    #[case("../../tests/inputs/webcast_jones_2021_judge_submission.zip")]
    #[case("../../tests/inputs/webcast_jones_08_2021.zip")]
    #[case("../../tests/inputs/webcast_jones_2021.zip")]
    #[case("../../tests/inputs/webcast_jones.zip")]
    #[case("../../tests/inputs/1a_fase_2021_frozen_unlocked_argentina.zip")]
    #[tokio::test]
    async fn test_golden_model_fast_tests(
        #[case] test_input: &str,
    ) -> color_eyre::eyre::Result<()> {
        check_revelation(test_input, &format!("{test_input}.revelation"))
            .await
            .inspect_err(|_err| eprint!("test: {test_input}"))
    }

    #[cfg(feature = "slow_tests")]
    #[rstest]
    #[case("../../tests/inputs/webcast.rinhadecalouros.zip")]
    #[case("../../tests/inputs/1a_fase_2021_frozen_unlocked.zip")]
    #[case("../../tests/inputs/2022/pre-warmup-webcast.zip")]
    #[case("../../tests/inputs/2022/brspso.zip")]
    #[case("../../tests/inputs/2a_fase_2022-23/countdown-test.zip")]
    #[case("../../tests/inputs/2a_fase_2022-23/global.zip")]
    #[case("../../tests/inputs/2a_fase_2022-23/ccl.zip")]
    #[case("../../tests/inputs/victor/webcast.zip")]
    #[case("../../tests/inputs/maratona-mineira-2024/t.zip")]
    #[case("../../tests/inputs/maratona-mineira-2024/t2.zip")]
    #[case("../../tests/inputs/maratona-mineira-2024/t1.zip")]
    #[case("../../tests/inputs/webcast_zip_1a_fase_2020.zip")]
    #[case("../../tests/inputs/webcast_frozen_aquecimento_1a_fase_2021.zip")]
    #[case("../../tests/inputs/webcast_frozen_aquecimento_1a_fase_2020.zip")]
    #[case("../../tests/inputs/webcast_1573336220.zip")]
    #[case("../../tests/inputs/warmup_2a_fase_2020_fake_frozen.zip")]
    #[case("../../tests/inputs/2a_fase_2023_chapeco/ccl.zip")]
    #[case("../../tests/inputs/webcast_early_frozen.zip")]
    #[case("../../tests/inputs/webcast_frozen_aquecimento_1a_fase_2021_frozen_unlocked.zip")]
    #[case("../../tests/inputs/2a_fase_2021-22/cafecomleite.zip")]
    #[case("../../tests/inputs/2a_fase_2021-22/brasil.zip")]
    // The tests that have been commented out have an X answer and don't have a good golden model
    // #[case("../../tests/inputs/webcast-x-answer.zip")]
    // #[case("../../tests/inputs/webcast-2023-1a-fase-meio-prova.zip")]
    // #[case("../../tests/inputs/pda-2024/pda-2024.zip")]
    // #[case("../../tests/inputs/webcast-2023-1a-fase-meio-prova-2.zip")]
    // #[case("../../tests/inputs/webcast-2023-1a-fase-final-prova.zip")]
    // #[case("../../tests/inputs/webcast-joao-2.zip")]
    #[tokio::test]
    async fn test_golden_model_slow_tests(
        #[case] test_input: &str,
    ) -> color_eyre::eyre::Result<()> {
        check_revelation(test_input, &format!("{test_input}.revelation"))
            .await
            .inspect_err(|_err| eprint!("test: {test_input}"))
    }
}
