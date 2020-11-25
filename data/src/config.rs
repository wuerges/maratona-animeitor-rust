use crate::configdata::*;

// const sedes :Vec<&str>= 
//     vec!["Global", "Brasil", "ac", "al", "am", "ap", "ba", "ce", "df", 
//         "es", "go", "ma", "mg", "ms", "mt", "pa", "pb", "pe", "pi", 
//         "pr", "rj","rn", "ro", "rr", "rs", "sc", "se", "sp", "to", 
//         "Scentrooeste", "Snordeste", "Snorte"];

const BR : &str = "Brasil";

pub fn contest() -> ConfigContest {
    let mut sedes : Vec<Sede> = 
        vec![("Acre", "ac", 0), 
             ("Alagoas", "al", 0), 
             ("Amazonas", "am", 1), 
             ("Amapá", "ap", 1), 
             ("Bahia", "ba", 2), 
             ("Ceará","ce", 3), 
             ("Distrito Federal","df", 1), 
             ("Espírito Santo", "es", 1),
             ("Goiás", "go", 1),
             ("Maranhão", "ma", 0), 
             ("Minas Gerais", "mg", 6), 
             ("Mato Grosso do Sul", "ms", 0), 
             ("Mato Grosso", "mt", 0), 
             ("Pará", "pa", 0), 
             ("Paraíba", "pb", 1), 
             ("Pernambuco", "pe", 0), 
             ("Piauí", "pi", 0), 
             ("Paraná", "pr", 2), 
             ("Rio de Janeiro", "rj", 2),
             ("Rio Grande do Norte", "rn", 0), 
             ("Rondônia", "ro", 0), 
             ("Roraima", "rr", 0), 
             ("Rio Grande do Sul", "rs", 1), 
             ("Santa Catarina", "sc", 3), 
             ("Sergipe", "se", 0), 
             ("São Paulo", "sp", 8), 
             ("Tocantins", "to", 0)]
            .iter()
            .map(|(n, s, v)| Sede::new(n, s, BR, format!("teambr{}", s).as_str(), *v > 0, *v))
            .collect();

    sedes.push(Sede::supersede("Supersede Centro-Oeste", "Scentrooeste", BR, vec!["teambrmt", "teambrms"], true, 1));
    
    sedes.push(Sede::supersede("Supersede Nordeste", "Snordeste", BR, 
        vec!["teambral", "teambrma", "teambrpe", "teambrrn", "teambrse"], true, 2));
    
    sedes.push(Sede::supersede("Supersede Norte", "Snorte", BR, 
        vec!["teambrac", "teambrpa", "teambrro", "teambrrr", "teambrto"], true, 2));

    sedes.push(Sede::new(BR, BR, "Global", "teambr", false, 0));

    // sedes.push(Sede::new("Global", "Global", "Global", ""));

    ConfigContest::new(sedes)
}