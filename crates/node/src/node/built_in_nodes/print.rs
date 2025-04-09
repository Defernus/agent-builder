use crate::*;
use eyre::ContextCompat;
use std::collections::BTreeMap;

/// Print text to console
pub struct NodePrint;

impl NodePrint {
    pub const INPUT_ARG_TEXT: &'static str = "text";
}

impl NodeTrait for NodePrint {
    fn run<'a>(
        &'a self,
        instance: &'a NodeInstance,
        _state: &'a Task,
        input: &'a InstanceRefArgs,
    ) -> RunResult<'a> {
        Box::pin(async {
            let text = *input
                .get(Self::INPUT_ARG_TEXT)
                .context("Print node: missing input argument")?;

            let text = text.downcast::<String>()?;
            println!("PrintNode {}:\n{}", instance.instance_id, text);

            Ok(BTreeMap::default())
        })
    }
}

impl NodeMetaTrait for NodePrint {
    fn get_meta(&self) -> NodeMeta {
        NodeMeta::new("print", "0.1.0")
            .with_input_arg(Self::INPUT_ARG_TEXT, InputArgMeta::new::<String>())
    }
}
