// Keep the Python bindings in one Rust module so PyO3 class registration and
// helper visibility stay simple, while splitting the large surface by domain.
include!("support.rs");
include!("timeline.rs");
include!("text.rs");
include!("ai.rs");
include!("shapes.rs");
include!("math.rs");
include!("axes_table.rs");
include!("three_d.rs");
include!("scene.rs");
include!("module.rs");
