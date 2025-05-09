use data::Letter;
use leptos::prelude::*;

fn number_submissions(s: usize) -> Option<usize> {
    if s == 1 {
        None
    } else {
        Some(s - 1)
    }
}

#[component]
pub fn Problem(prob: Letter, problem: Signal<Option<data::ProblemView>>) -> impl IntoView {
    let problem_class = move || {
        problem.with(|p| match &p {
            Some(p) => {
                if p.solved && p.solved_first {
                    "star cell quadrado".to_string()
                } else if p.solved {
                    "accept cell quadrado".to_string()
                } else {
                    let cell_type = if p.pending > 0 { "inqueue" } else { "unsolved" };
                    format!("cell quadrado {cell_type}")
                }
            }
            None => "not-tried cell quadrado".to_string(),
        })
    };

    let problem_content = move || {
        problem.with(|problem| match problem {
            Some(p) => (if p.solved {
                let balao = format!("balao_{}", prob);
                let img = if p.solved_first {
                    "star-img"
                } else {
                    "accept-img"
                };
                view! {
                    <div class=format!("{img} {balao}")></div>
                    <div class="accept-text cell-content">
                        <div class="cima">
                            +{number_submissions(p.submissions)}
                        </div>
                        <div class="baixo">
                            {p.time_solved}
                        </div>
                    </div>
                }
                .into_any()
            } else {
                let pending = match p.pending {
                    0 => "X".to_string(),
                    1 => "?".to_string(),
                    n => format!("({})", n),
                };

                view! {
                    <div class="cima">{pending}</div>
                    <div class="baixo"> +{p.submissions}" "</div>
                }
                .into_any()
            })
            .into_any(),
            None => { "-" }.into_any(),
        })
    };

    view! {
        <div class={problem_class}>
            {problem_content}
        </div>
    }
}
