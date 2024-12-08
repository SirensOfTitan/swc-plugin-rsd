use std::path::PathBuf;

use swc_core::ecma::parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::{FixtureTestConfig, test_fixture};
use swc_plugin_rsd::plugin::react_strict_dom_plugin;

#[testing::fixture("tests/fixtures/**/*.tsx")]
fn fixture_typescript_simple(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..TsSyntax::default()
        }),
        &|config| react_strict_dom_plugin(true, config.cm.clone()),
        &input,
        &output,
        FixtureTestConfig {
            allow_error: true,
            sourcemap: true,
            module: Some(true),
        },
    )
}
