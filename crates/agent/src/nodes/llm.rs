use eyre::ContextCompat;
use node::*;
use std::collections::BTreeMap;

pub struct NodeLLM {}

impl NodeLLM {
    pub const INPUT_ARG_CONTEXT: &str = "context";
    pub const OUTPUT_ARG_TEXT: &str = "text";
}

impl NodeTrait for NodeLLM {
    fn run<'a>(
        &'a self,
        _instance: &'a NodeInstance,
        _state: &'a Task,
        input: &'a InstanceRefArgs,
    ) -> RunResult<'a> {
        Box::pin(async {
            let _context = *input
                .get(Self::INPUT_ARG_CONTEXT)
                .context("Print node: missing input argument")?;

            Ok(BTreeMap::default())
        })
    }
}

impl NodeMetaTrait for NodeLLM {
    fn get_meta(&self) -> NodeMeta {
        NodeMeta::new("llm", "0.1.0")
            .with_input_arg(Self::INPUT_ARG_CONTEXT, InputArgMeta::new::<String>())
            .with_output_arg(Self::OUTPUT_ARG_TEXT, OutputArgMeta::new::<String>())
    }
}
