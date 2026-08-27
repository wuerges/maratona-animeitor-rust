use itertools::Itertools;
use std::sync::LazyLock;

use data::Letter;

static ALPHABET: LazyLock<Vec<char>> =
    LazyLock::new(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect());

static CALCULATED: LazyLock<Vec<Letter>> = LazyLock::new(|| {
    (1..=3)
        .flat_map(|l| {
            ALPHABET
                .iter()
                .combinations_with_replacement(l)
                .map(|t| t.into_iter().collect::<String>().parse().expect("valid letter"))
                .collect_vec()
        })
        .sorted()
        .collect_vec()
});

pub fn problem_letters(i: usize) -> Vec<Letter> {
    CALCULATED.iter().take(i).cloned().collect_vec()
}

#[cfg(test)]
mod tests {
    use super::problem_letters;

    #[test]
    fn check_first_letters() {
        let letters = problem_letters(30);

        let ordered = vec![
            "A".parse().unwrap(),
            "B".parse().unwrap(),
            "C".parse().unwrap(),
            "D".parse().unwrap(),
            "E".parse().unwrap(),
            "F".parse().unwrap(),
            "G".parse().unwrap(),
            "H".parse().unwrap(),
            "I".parse().unwrap(),
            "J".parse().unwrap(),
            "K".parse().unwrap(),
            "L".parse().unwrap(),
            "M".parse().unwrap(),
            "N".parse().unwrap(),
            "O".parse().unwrap(),
            "P".parse().unwrap(),
            "Q".parse().unwrap(),
            "R".parse().unwrap(),
            "S".parse().unwrap(),
            "T".parse().unwrap(),
            "U".parse().unwrap(),
            "V".parse().unwrap(),
            "W".parse().unwrap(),
            "X".parse().unwrap(),
            "Y".parse().unwrap(),
            "Z".parse().unwrap(),
            "AA".parse().unwrap(),
            "AB".parse().unwrap(),
            "AC".parse().unwrap(),
            "AD".parse().unwrap(),
        ];

        assert_eq!(letters, ordered)
    }
}
