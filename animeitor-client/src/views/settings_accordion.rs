use leptos::prelude::*;

use super::global_settings::SettingsPanel;

/// Native disclosure keeps the form and any active export mounted when closed.
#[component]
pub fn SettingsAccordion(
    #[prop(default = true)] show_secret: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let details = NodeRef::<leptos::html::Details>::new();
    let summary = NodeRef::<leptos::html::Summary>::new();

    view! {
        <details
            class="settings-accordion"
            node_ref=details
            on:keydown=move |event: web_sys::KeyboardEvent| {
                // Editing settings must not reveal runs or trigger media keys.
                event.stop_propagation();
                if event.key() == "Escape" {
                    event.prevent_default();
                    if let Some(details) = details.get() {
                        let _ = details.remove_attribute("open");
                    }
                    if let Some(summary) = summary.get() {
                        let _ = summary.focus();
                    }
                }
            }
        >
            <summary node_ref=summary>"Settings"</summary>
            <div class="settings-accordion-content">
                <SettingsPanel show_secret />
                {children.map(|children| children())}
            </div>
        </details>
    }
}
