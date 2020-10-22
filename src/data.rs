
enum Answer {
    Yes,
    No,
    Unk
}

pub struct Team {
    pub name : String,
    pub score : i64
}

struct RunTuple {
    id : i64,
    time : i64,
    team : String,
    prob : String,
    answer : Answer
}

