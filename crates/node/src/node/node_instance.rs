use crate::*;
use eyre::Context;
use std::collections::BTreeMap;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::Deref,
    derive_more::Into,
    derive_more::From,
)]
pub struct NodeInstanceId(pub u32);

#[derive(Debug)]
pub struct NodeInstanceIdProvider {
    pub next_id: u32,
}

impl Default for NodeInstanceIdProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeInstanceIdProvider {
    const START_ID: u32 = 10_000;

    pub fn new() -> Self {
        Self {
            next_id: Self::START_ID,
        }
    }

    pub fn next_id(&mut self) -> NodeInstanceId {
        let id = self.next_id;
        self.next_id += 1;
        NodeInstanceId(id)
    }
}

#[derive(Clone, Debug)]
pub struct NodeConnection {
    pub instance: NodeInstanceId,
    pub arg_name: String,
}

pub struct NodeInstance {
    pub node_id: NodeId,
    pub instance_id: NodeInstanceId,
    pub memory: BTreeMap<String, Value>,
    pub input_connections: BTreeMap<String, NodeConnection>,
    pub output_connections: BTreeMap<String, NodeConnection>,
}

impl NodeInstance {
    pub fn new(node: &Node, id: NodeInstanceId) -> Self {
        Self {
            instance_id: id,
            node_id: node.id().clone(),
            memory: BTreeMap::new(),
            input_connections: BTreeMap::new(),
            output_connections: BTreeMap::new(),
        }
    }

    /// If node does not have any output connections, it is a leaf node.
    pub fn is_leaf(&self) -> bool {
        self.output_connections.is_empty()
    }

    #[tracing::instrument(
        skip_all,
        fields(instance_id = ?self.instance_id, name, type_name = std::any::type_name::<T>())
    )]
    pub fn set_memory<T: ValueTrait>(&mut self, name: String, val: T) -> eyre::Result<()> {
        self.memory.insert(name, Value::new(val));

        Ok(())
    }

    #[tracing::instrument(
        skip_all,
        fields(instance_id = ?self.instance_id, name, type_name = std::any::type_name::<T>())
    )]
    pub fn get_memory<T: ValueTrait>(&self, name: &str) -> eyre::Result<Option<&T>> {
        self.memory
            .get(name)
            .map(Value::downcast)
            .transpose()
            .context("Failed to downcast value to the type")
    }
}
