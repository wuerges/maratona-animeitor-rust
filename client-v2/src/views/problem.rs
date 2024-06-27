use leptos::*;

fn number_submissions(s: usize) -> Option<usize> {
    if s == 1 {
        None
    } else {
        Some(s - 1)
    }
}

#[component]
pub fn Problem(prob: char, problem: Option<data::ProblemView>) -> impl IntoView {
    // log!("rendered problem {:?}", problem);
    view! {
            <div class={match &problem {
                Some(p) => if p.solved && p.solved_first {
                    "star cell quadrado".to_string()
                } else if p.solved {
                    "accept cell quadrado".to_string()
                } else {
                    let cell_type = if p.wait { "inqueue"} else { "unsolved" };
                    format!("cell quadrado {cell_type}")
                },
                None => "not-tried cell quadrado".to_string(),
            }}>
            {match &problem {
                Some(p) => {
                    (if p.solved {
                        let balao = format!("balao_{}", prob);
                        let img = if p.solved_first { "star-img"} else { "accept-img" };
                        view! {
                            <div class=format!("{img} {balao}")></div>
                            <div class="accept-text cell-content">
                                +{number_submissions(p.submissions)}<br />{p.time_solved}
                            </div>
                        }
                    } else {
                        let cell_symbol = if p.wait { "?" } else { "X" };

                        view! {
                            <div class="cima">{cell_symbol}</div>
                            <div class="baixo">"("{p.submissions}")"</div>
                        }
                    }).into_view()

                },
                None => {
                    {"-"}.into_view()
                },
            }}
            </div>
    }
}
