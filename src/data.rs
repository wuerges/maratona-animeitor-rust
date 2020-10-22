use std::io::{self, Read};
use std::fs::File;
use std::{error::Error, fmt};


#[derive(Debug, PartialEq)]
enum Answer {
    Yes,
    No,
    Wait,
    Unk
}

#[derive(Debug)]
enum ContestError {
    IO(io::Error),
    Parse(std::num::ParseIntError),
    Simple(String)
}

impl Error for ContestError {}

impl fmt::Display for ContestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Answer could not be parsed: {:?}", self)
    }
}

impl Answer {
    fn from_string(t : &str) -> Result<Answer, ContestError> {
        match t {
            "Y" => Ok(Self::Yes),
            "N" => Ok(Self::No),
            "?" => Ok(Self::Wait),
            _ => Err(ContestError::Simple(t.to_string()))
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
    pub fn from_string(line : &str) -> Result<Self, ContestError> {
        let v : Vec<&str> = line.split('').collect();
        let id = v[0].parse().map_err(|e| ContestError::Parse(e))?;
        let time = v[1].parse().map_err(|e| ContestError::Parse(e))?;
        let ans = Answer::from_string(v[4])?;
        
        Ok(Self {
            id   : id,
            time : time,
            team : v[2].to_string(),
            prob : v[3].to_string(),
            answer : ans
        })
    }
    
    fn from_file(s: &str) -> Result<Vec<Self>, ContestError> {
        let mut file = File::open(s).map_err(|e| ContestError::IO(e))?;
        let mut s = String::new();
        
        file.read_to_string(&mut s).map_err(|e| ContestError::IO(e))?;
        
        s.lines().map( |line| Self::from_string(line) ).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_string() -> Result<(), ContestError> {
        let x = "375971416299teambrbr3BN";
        let t = RunTuple::from_string(x)?;

        assert_eq!(t.id, 375971416);
        assert_eq!(t.time, 299);
        assert_eq!(t.team, "teambrbr3");
        assert_eq!(t.prob, "B");
        assert_eq!(t.answer, Answer::No);
        Ok(())
    }

    #[test]
    fn test_parse_file() -> Result<(), ContestError> {
        let x = RunTuple::from_file("test/sample/runs")?;
        assert_eq!(x.len(), 716);
        Ok(())
    }
}