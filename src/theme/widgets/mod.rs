//! The custom-painted widgets. Each one lays its own text out (never `Painter::text` on a
//! variable-width string) so nothing can be painted over a neighbour, and each grows taller
//! rather than cutting a word off when the column it is given is narrow.

mod card;
mod console;
mod dial;
mod gauge;
mod nav;
mod status;
mod text;

pub use card::{action_card_button, cyber_card, mode_selector_card, module_toggle_card};
pub use console::cyber_console;
pub use dial::radial_dial;
pub use gauge::{mini_core_bar, resource_gauge_card};
pub use nav::nav_sidebar_item;
pub use status::{legend_strip, status_pill};
