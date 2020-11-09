use crate::configdata::*;

// const sedes :Vec<&str>= 
//     vec!["Global", "Brasil", "ac", "al", "am", "ap", "ba", "ce", "df", 
//         "es", "go", "ma", "mg", "ms", "mt", "pa", "pb", "pe", "pi", 
//         "pr", "rj","rn", "ro", "rr", "rs", "sc", "se", "sp", "to", 
//         "Scentrooeste", "Snordeste", "Snorte"];

const BR : &str = "Brasil";

pub fn contest() -> Contest {
    let mut sedes : Vec<Sede> = 
        vec![("Acre", "ac"), 
             ("Alagoas", "al"), 
             ("Amazonas", "am"), 
             ("Amapá", "ap"), 
             ("Bahia", "ba"), 
             ("Ceará","ce"), 
             ("Distrito Federal","df"), 
             ("Espírito Santo", "es"),
             ("Goiás", "go"),
             ("Maranhão", "ma"), 
             ("Minas Gerais", "mg"), 
             ("Mato Grosso do Sul", "ms"), 
             ("Mato Grosso", "mt"), 
             ("Pará", "pa"), 
             ("Paraíba", "pb"), 
             ("Pernambuco", "pe"), 
             ("Piauí", "pi"), 
             ("Paraná", "pr"), 
             ("Rio de Janeiro", "rj"),
             ("Rio Grande do Norte", "rn"), 
             ("Rondônia", "ro"), 
             ("Roraima", "rr"), 
             ("Rio Grande do Sul", "rs"), 
             ("Santa Catarina", "sc"), 
             ("Sergipe", "se"), 
             ("São Paulo", "sp"), 
             ("Tocantins", "to")]
            .iter()
            .map(|(n, s)| Sede::new(n, s, BR, format!("teambr{}", s).as_str()))
            .collect();

    sedes.push(Sede::supersede("Supersede Centro-Oeste", "Scentrooeste", BR, vec!["teambrmt", "teambrms"]));
    
    sedes.push(Sede::supersede("Supersede Nordeste", "Snordeste", BR, 
        vec!["teambral", "teambrma", "teambrpe", "teambrrn", "teambrse"]));
    
    sedes.push(Sede::supersede("Supersede Norte", "Snorte", BR, 
        vec!["teambrac", "teambrap", "teambrpa", "teambrro", "teambrrr", "teambrto"]));

    sedes.push(Sede::new(BR, BR, "Global", "teambr"));

    sedes.push(Sede::new("Global", "Global", "Global", ""));

    Contest::new(sedes)
}