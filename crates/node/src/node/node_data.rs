use crate::*;
use eyre::ContextCompat;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

pub type InstanceArgs = BTreeMap<String, Value>;
pub type InstanceRefArgs<'a> = BTreeMap<&'a str, &'a Value>;

pub type RunResult<'a> = Pin<Box<dyn Future<Output = eyre::Result<InstanceArgs>> + 'a>>;

pub trait NodeTrait: Send + Sync + 'static {
    fn run<'a>(
        &'a self,
        instance: &'a NodeInstance,
        state: &'a Task,
        input: &'a InstanceRefArgs,
    ) -> RunResult<'a>;
}

pub struct Node {
    inner: Box<dyn NodeTrait>,
    meta: NodeMeta,
}

#[derive(
    Clone,
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
pub struct NodeId(pub String);

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub struct NodeMeta {
    pub id: NodeId,
    pub version: String,
    pub input_args: BTreeMap<String, InputArgMeta>,
    pub output_args: BTreeMap<String, OutputArgMeta>,
}

impl NodeMeta {
    pub fn new(id: impl Into<NodeId>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            input_args: BTreeMap::new(),
            output_args: BTreeMap::new(),
        }
    }

    pub fn with_input_arg(mut self, key: impl Into<String>, value: InputArgMeta) -> Self {
        self.input_args.insert(key.into(), value);
        self
    }

    pub fn with_output_arg(mut self, key: impl Into<String>, value: OutputArgMeta) -> Self {
        self.output_args.insert(key.into(), value);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputArgMeta {
    pub value_type: ValueType,
}

impl OutputArgMeta {
    pub fn new<T: std::any::Any>() -> Self {
        Self {
            value_type: ValueType::new::<T>(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputArgMeta {
    pub value_type: ValueType,
    pub is_optional: bool,
}

impl InputArgMeta {
    pub fn new<T: std::any::Any>() -> Self {
        Self {
            value_type: ValueType::new::<T>(),
            is_optional: false,
        }
    }

    pub fn with_optional(mut self, is_optional: bool) -> Self {
        self.is_optional = is_optional;
        self
    }
}

pub trait NodeMetaTrait {
    fn get_meta(&self) -> NodeMeta;
}

impl Node {
    #[inline]
    pub fn new_with_meta<T: NodeTrait>(node: T, meta: NodeMeta) -> Self {
        Self {
            inner: Box::new(node),
            meta,
        }
    }

    #[inline]
    pub fn new<T: NodeTrait + NodeMetaTrait>(node: T) -> Self {
        let meta = node.get_meta();

        Self::new_with_meta(node, meta)
    }

    #[inline]
    pub fn get_meta(&self) -> &NodeMeta {
        &self.meta
    }

    #[inline]
    #[tracing::instrument(skip_all, fields(node_id = ?self.meta.id))]
    pub async fn run<'a>(
        &self,
        instance: &NodeInstance,
        task: &Task,
        input: &InstanceRefArgs<'a>,
    ) -> eyre::Result<InstanceArgs> {
        self.inner.run(instance, task, input).await
    }

    pub fn id(&self) -> &NodeId {
        &self.meta.id
    }

    #[tracing::instrument(skip(self), fields(node_id = ?self.meta.id))]
    pub fn get_input_arg(&self, key: &str) -> eyre::Result<InputArgMeta> {
        self.meta
            .input_args
            .get(key)
            .context("Input argument not found")
            .cloned()
    }

    #[tracing::instrument(skip(self), fields(node_id = ?self.meta.id))]
    pub fn get_out_arg(&self, key: &str) -> eyre::Result<OutputArgMeta> {
        self.meta
            .output_args
            .get(key)
            .context("Output argument not found")
            .cloned()
    }

    pub fn input_args(&self) -> &BTreeMap<String, InputArgMeta> {
        &self.meta.input_args
    }

    pub fn output_args(&self) -> &BTreeMap<String, OutputArgMeta> {
        &self.meta.output_args
    }

    /// If node does not have any input arguments, it is a root node.
    pub fn is_root(&self) -> bool {
        self.meta.input_args.is_empty()
    }
}

impl<T: NodeTrait + NodeMetaTrait> From<T> for Node {
    fn from(node: T) -> Self {
        Self::new(node)
    }
}
