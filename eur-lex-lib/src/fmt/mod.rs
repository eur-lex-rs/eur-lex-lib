//! Output formatters for [`crate::model::Act`], enabled by the `fmt` Cargo feature.
//!
//! Three formats are provided:
//!
//! - [`markdown`] — GitHub-flavoured Markdown
//! - [`txt`] — Plain text with no markup syntax
//! - [`html`] — Self-contained HTML5 document
//!
//! All modules expose a single public function: `render_act(act: &Act) -> String`.

pub mod html;
pub mod markdown;
pub mod txt;
