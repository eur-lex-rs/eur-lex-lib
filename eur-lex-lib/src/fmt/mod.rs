//! Output formatters for [`crate::model::Act`], enabled by the `fmt` Cargo feature.
//!
//! Two formats are provided:
//!
//! - [`markdown`] — GitHub-flavoured Markdown
//! - [`txt`] — Plain text with no markup syntax
//!
//! Both modules expose a single public function: `render_act(act: &Act) -> String`.

pub mod markdown;
pub mod txt;
