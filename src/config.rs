use crate::configdata::*;

// const sedes :Vec<&str>= 
//     vec!["Global", "Brasil", "ac", "al", "am", "ap", "ba", "ce", "df", 
//         "es", "go", "ma", "mg", "ms", "mt", "pa", "pb", "pe", "pi", 
//         "pr", "rj","rn", "ro", "rr", "rs", "sc", "se", "sp", "to", 
//         "Scentrooeste", "Snordeste", "Snorte"];

const BR : &str = "Brasil";

pub fn contest() -> Contest {
    let mut sedes : Vec<Sede> = 
        vec!["ac", "al", "am", "ap", "ba", "ce", "df", 
            "es", "go", "ma", "mg", "ms", "mt", "pa", "pb", "pe", "pi", 
            "pr", "rj","rn", "ro", "rr", "rs", "sc", "se", "sp", "to"]
            .iter()
            .map(|s| Sede::new(s, s, BR, format!("teambr{}", s).as_str()))
            .collect();

    sedes.push(Sede::supersede("Scentrooeste", "Scentrooeste", BR, vec!["teambrmt", "teambrms"]));
    
    sedes.push(Sede::supersede("Snordeste", "Snordeste", BR, 
        vec!["teambral", "teambrma", "teambrpe", "teambrrn", "teambrse"]));
    
    sedes.push(Sede::supersede("Snorte", "Snorte", BR, 
        vec!["teambrac", "teambrap", "teambrpa", "teambrro", "teambrrr", "teambrto"]));

    sedes.push(Sede::new(BR, BR, "Global", "teambr"));

    sedes.push(Sede::new("Global", "Global", "Global", ""));

    Contest::new(
        "localhost", 
        sedes)
}