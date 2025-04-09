use agent::*;
use init_log::init_logging;
use std::sync::Arc;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    init_logging(true);

    let _config = Arc::new(Config::init()?);

    let mut task = node::Task::new();

    let node_text = task.register_node(node::NodeText)?;
    let node_print = task.register_node(node::NodePrint)?;

    let node_text = task.instantiate(&node_text)?;
    let node_print = task.instantiate(&node_print)?;

    task.set_instance_memory(node_text, "text", "Test!".to_string())?;

    task.connect(node_text, "text", node_print, "text")?;

    task.run().await?;

    Ok(())
}
