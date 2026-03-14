/// Package browser — grid of available packages from the FSN store.
use dioxus::prelude::*;

use crate::package_card::{PackageCard, PackageEntry};

/// Package browser component — fetches and renders available packages.
#[component]
pub fn PackageBrowser(search: String) -> Element {
    // TODO: load from fsn-store via use_resource
    let packages = use_signal(Vec::<PackageEntry>::new);

    rsx! {
        div {
            class: "fsd-browser",

            if packages.read().is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-color-text-muted); padding: 48px;",
                    p { "No packages found." }
                    p { "Make sure you have an internet connection or check your store URL in Settings." }
                }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px;",
                    for pkg in packages.read().iter() {
                        PackageCard { package: pkg.clone() }
                    }
                }
            }
        }
    }
}
