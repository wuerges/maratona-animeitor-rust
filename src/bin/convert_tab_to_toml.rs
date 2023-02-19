use data::configdata::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    for a in args.iter().skip(1) {
        // println!("args: {}", a);
        let f = std::fs::read_to_string(a)?;
        // let teams : ConfigTeams = toml::from_str(&f)?;

        // pub login: String,
        // pub nome: Option<String>,
        // pub foto: Option<String>,
        // pub musica: Option<String>,
        // pub comentario: Option<String>

        let out: Vec<TeamEntry> = f
            .lines()
            .map(|l| {
                let mut fields = l.split('\t');
                let login = fields.next().unwrap();
                let nome = fields.next().unwrap();
                TeamEntry {
                    login: login.to_string(),
                    nome: Some(nome.to_string()),
                    foto: Some(format!("{}.png", login)),
                    musica: None,
                    comentario: None,
                }
            })
            .collect();

        let teams = ConfigTeams { teams: out };

        let result = toml::to_string(&teams).unwrap();
        println!("-------------");
        println!("{}", result);
    }

    Ok(())
}
