
#[derive(Debug, PartialEq)]
enum Answer {
    Yes,
    No,
    Wait,
    Unk
}

impl Answer {
    fn from_string(t : &str) -> Self {
        match t {
            "Y" => Self::Yes,
            "N" => Self::No,
            "?" => Self::Wait,
            _ => Self::Unk
        }        
    }
}

#[derive(Debug)]
pub struct Team {
    pub name : String,
    pub score : i64
}

#[derive(Debug)]
struct RunTuple {
    id : i64,
    time : i64,
    team : String,
    prob : String,
    answer : Answer
}

impl RunTuple {
    pub fn from_string(line : &str) -> Self {
        let v : Vec<&str> = line.split('').collect();
        Self {
            id   : v[0].parse().unwrap(),
            time : v[1].parse().unwrap(),
            team : v[2].to_string(),
            prob : v[3].to_string(),
            answer : Answer::from_string(v[4])
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_string() {
        let x = "375971416299teambrbr3BN";
        let t = RunTuple::from_string(x);

        assert_eq!(t.id, 375971416);
        assert_eq!(t.time, 299);
        assert_eq!(t.team, "teambrbr3");
        assert_eq!(t.prob, "B");
        assert_eq!(t.answer, Answer::No);
    }
}