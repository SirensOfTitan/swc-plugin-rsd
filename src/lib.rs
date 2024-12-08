pub mod plugin;

use std::sync::Arc;

use serde::Deserialize;
use swc_core::{
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReactStrictDomConfig {
    /// Enables debug mode, which will add an "html" source map to each RSD element
    debug: bool,
}

#[plugin_transform]
fn rsd_plugin_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let plugin_config: ReactStrictDomConfig = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .unwrap_or("{}".to_string()),
    )
    .expect("Should provide valid config.");

    let react_strict_dom_pass =
        plugin::react_strict_dom_plugin(plugin_config.debug, Arc::new(metadata.source_map));
    program.apply(react_strict_dom_pass)
}
