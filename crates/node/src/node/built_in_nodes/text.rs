use crate::*;
use std::collections::BTreeMap;

/// Use to provide text input to other nodes
pub struct NodeText;

impl NodeText {
    pub const OUT_ARG_TEXT: &str = "text";
    pub const MEMORY_TEXT: &str = "text";
}

impl NodeTrait for NodeText {
    fn run<'a>(
        &'a self,
        instance: &'a NodeInstance,
        _state: &'a Task,
        _input: &'a InstanceRefArgs,
    ) -> RunResult<'a> {
        Box::pin(async {
            let text_value = instance
                .get_memory::<String>(Self::MEMORY_TEXT)?
                .cloned()
                .unwrap_or_default();

            Ok(BTreeMap::from([(
                Self::OUT_ARG_TEXT.to_string(),
                Value::new(text_value),
            )]))
        })
    }
}

impl NodeMetaTrait for NodeText {
    fn get_meta(&self) -> NodeMeta {
        NodeMeta::new("text", "0.1.0")
            .with_output_arg(Self::OUT_ARG_TEXT, OutputArgMeta::new::<String>())
    }
}
