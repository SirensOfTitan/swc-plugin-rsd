mod plugin;

use serde::Deserialize;
use swc_core::ecma::ast::Program;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReactStrictDomConfig {
    // TODO:
}

#[plugin_transform]
fn rsd_plugin_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let _plugin_config: ReactStrictDomConfig = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .unwrap_or("{}".to_string()),
    )
    .expect("Should provide valid config.");

    let react_strict_dom_pass = plugin::react_strict_dom_plugin();
    program.apply(react_strict_dom_pass)
}
