use std::borrow::Cow;

use leptos::prelude::*;

use crate::model::animeitor_v3::team::ProblemState;

#[component]
pub fn Problem(letter: String, problem: Signal<ProblemState>) -> impl IntoView {
    let problem_class = move || match problem.get() {
        ProblemState::Fresh => "not-tried cell quadrado",
        ProblemState::UnderJudgement { .. } => "inqueue cell quadrado",
        ProblemState::Solved { is_first, .. } => {
            if is_first {
                "star cell quadrado"
            } else {
                "accept cell quadrado"
            }
        }
        ProblemState::WrongAnswer { .. } => "unsolved cell quadrado",
    };

    let balao = format!("balao_{}", letter);

    let problem_content = move || match problem.get() {
        ProblemState::Fresh => "-".into_any(),
        ProblemState::UnderJudgement {
            failed_attempts,
            new_attempts,
        } => {
            let pending = match new_attempts {
                0 => Cow::Borrowed("X"),
                1 => Cow::Borrowed("?"),
                n => format!("({n})").into(),
            };
            view! {
                <div class="cima">{pending}</div>
                <div class="baixo"> +{failed_attempts}" "</div>
            }
            .into_any()
        }
        ProblemState::Solved {
            is_first,
            time_in_minutes,
            penalty: _,
            attempts,
        } => {
            let img_class = if is_first { "star-img" } else { "accept-img" };
            let submissions = (attempts > 1).then_some(attempts - 1);
            view! {
                <div class=format!("{balao} {img_class}")></div>
                    <div class="accept-text cell-content">
                        <div class="cima">
                            +{submissions}
                        </div>
                        <div class="baixo">
                            {time_in_minutes}
                        </div>
                    </div>
            }
            .into_any()
        }
        ProblemState::WrongAnswer { judged_attempts } => view! {
            <div class="cima">X</div>
            <div class="baixo"> +{judged_attempts}" "</div>
        }
        .into_any(),
    };

    view! {
        <div class={problem_class}>
            {problem_content}
        </div>
    }
}
