use crate::*;
use eyre::ContextCompat;
use std::collections::{HashMap, HashSet};

/// Collection of nodes and connections between them.
pub struct Task {
    nodes: HashMap<NodeId, Node>,
    instances: HashMap<NodeInstanceId, NodeInstance>,
    instance_id_provider: NodeInstanceIdProvider,
}

impl Task {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            instance_id_provider: NodeInstanceIdProvider::default(),
            instances: HashMap::new(),
        }
    }

    pub fn register_node(&mut self, node: impl Into<Node>) -> eyre::Result<NodeId> {
        let node: Node = node.into();
        let id = node.id().clone();

        let _span = tracing::info_span!("register_node", id = %id).entered();

        match self.nodes.entry(id) {
            std::collections::hash_map::Entry::Occupied(_) => Err(eyre::eyre!("Id already exists")),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let id = entry.key().clone();
                entry.insert(node);

                Ok(id)
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn instantiate(&mut self, node_id: &NodeId) -> eyre::Result<NodeInstanceId> {
        let instance_id = self.instance_id_provider.next_id();

        let node = self.nodes.get(node_id).context("Node not found")?;
        let instance = NodeInstance::new(node, instance_id);

        self.instances.insert(instance_id, instance);

        Ok(instance_id)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_instance(&self, id: NodeInstanceId) -> eyre::Result<&NodeInstance> {
        self.instances.get(&id).context("Instance not found")
    }

    #[tracing::instrument(skip(self))]
    fn get_instance_mut(&mut self, id: NodeInstanceId) -> eyre::Result<&mut NodeInstance> {
        self.instances.get_mut(&id).context("Instance not found")
    }

    #[tracing::instrument(
        skip_all,
        fields(instance_id, name, type_name = std::any::type_name::<T>())
    )]
    pub fn set_instance_memory<T: ValueTrait>(
        &mut self,
        id: NodeInstanceId,
        name: impl Into<String>,
        val: T,
    ) -> eyre::Result<()> {
        let instance = self.get_instance_mut(id)?;
        instance.set_memory(name.into(), val)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_node(&self, id: &NodeId) -> eyre::Result<&Node> {
        self.nodes.get(id).context("Node not found")
    }

    #[tracing::instrument(skip(self))]
    pub fn connect(
        &mut self,
        output_id: NodeInstanceId,
        output_arg: &str,
        input_id: NodeInstanceId,
        input_arg: &str,
    ) -> eyre::Result<()> {
        if !self.can_connect_nodes(output_id, output_arg, input_id, input_arg)? {
            return Err(eyre::eyre!("Incompatible types"));
        }

        self.get_instance_mut(output_id)?.output_connections.insert(
            output_arg.to_string(),
            NodeConnection {
                instance: input_id.clone(),
                arg_name: input_arg.to_string(),
            },
        );
        self.get_instance_mut(input_id)?.input_connections.insert(
            input_arg.to_string(),
            NodeConnection {
                instance: output_id.clone(),
                arg_name: output_arg.to_string(),
            },
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn can_connect_nodes(
        &self,
        output_id: NodeInstanceId,
        output_arg: &str,
        input_id: NodeInstanceId,
        input_arg: &str,
    ) -> eyre::Result<bool> {
        Ok(
            self.match_types(output_id, output_arg, input_id, input_arg)?
                && !self.get_all_deps(output_id)?.contains(&input_id),
        )
    }

    /// Check if the output type of the node is compatible with the input type of the other node.
    #[tracing::instrument(skip(self))]
    pub fn match_types(
        &self,
        output_id: NodeInstanceId,
        output_arg: &str,
        input_id: NodeInstanceId,
        input_arg: &str,
    ) -> eyre::Result<bool> {
        let output_instance = self.get_instance(output_id)?;
        let output_node = self.get_node(&output_instance.node_id)?;

        let input_instance = self.get_instance(input_id)?;
        let input_node = self.get_node(&input_instance.node_id)?;

        let out_ty = output_node.get_out_arg(&output_arg)?;
        let in_ty = input_node.get_input_arg(&input_arg)?;

        Ok(in_ty.value_type == out_ty.value_type)
    }

    /// Get direct dependencies of a node.
    #[tracing::instrument(skip(self))]
    pub fn get_direct_deps(
        &self,
        instance: NodeInstanceId,
    ) -> eyre::Result<HashSet<NodeInstanceId>> {
        let instance = self.get_instance(instance)?;

        Ok(instance
            .input_connections
            .values()
            .map(|conn| conn.instance)
            .collect())
    }

    /// Get direct and indirect dependencies of a node.
    #[tracing::instrument(skip(self))]
    pub fn get_all_deps(&self, instance: NodeInstanceId) -> eyre::Result<HashSet<NodeInstanceId>> {
        let mut visited = HashSet::new();
        let mut stack = vec![instance];

        while let Some(instance) = stack.pop() {
            if visited.contains(&instance) {
                continue;
            }

            visited.insert(instance);

            let deps = self.get_direct_deps(instance)?;
            for dep in deps {
                stack.push(dep);
            }
        }

        Ok(visited)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_node_out_connection<'a>(
        &'a self,
        instance_id: NodeInstanceId,
        output_arg: &str,
    ) -> eyre::Result<Option<&'a NodeConnection>> {
        let instance = self.get_instance(instance_id)?;
        let Some(output_connected_to) = instance.output_connections.get(output_arg) else {
            return Ok(None);
        };

        Ok(Some(output_connected_to))
    }

    #[tracing::instrument(skip(self))]
    pub fn get_node_in_connection<'a>(
        &'a self,
        node_id: NodeInstanceId,
        input_arg: &str,
    ) -> eyre::Result<Option<&'a NodeConnection>> {
        let instance = self.get_instance(node_id)?;
        let Some(input_connected_to) = instance.input_connections.get(input_arg) else {
            return Ok(None);
        };

        Ok(Some(input_connected_to))
    }

    #[tracing::instrument(skip(self))]
    pub fn is_nodes_connected(
        &self,
        output_id: NodeInstanceId,
        output_arg: &str,
        input_id: NodeInstanceId,
        input_arg: &str,
    ) -> eyre::Result<bool> {
        let Some(output_connected_to) = self.get_node_out_connection(output_id, output_arg)? else {
            return Ok(false);
        };

        let Some(input_connected_to) = self.get_node_in_connection(input_id, input_arg)? else {
            return Ok(false);
        };

        let a_to_b =
            output_connected_to.instance == input_id && output_connected_to.arg_name == input_arg;
        let b_to_a =
            input_connected_to.instance == output_id && input_connected_to.arg_name == output_arg;

        match (a_to_b, b_to_a) {
            (true, true) => Ok(true),
            (false, false) => Ok(false),
            // If one node is connected to the other, but not the other way around,
            // we have a corrupted connection
            _ => Err(eyre::eyre!(
                "Corrupted connection: {}[{:?}] -> {}[{:?}] and {}[{:?}] -> {}[{:?}]",
                output_id,
                output_arg,
                output_connected_to.instance,
                output_connected_to.arg_name,
                input_id,
                input_arg,
                input_connected_to.instance,
                input_connected_to.arg_name
            )),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(&self) -> eyre::Result<()> {
        self.is_all_nodes_connected()?;

        let instances_to_update = self.get_leaf_nodes();
        let mut results = HashMap::<NodeInstanceId, InstanceArgs>::new();

        for instance in instances_to_update {
            self.update_node_recursive(instance, &mut results).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(instance_id = %instance.instance_id, results_len = %results.len()))]
    pub async fn update_node_recursive(
        &self,
        instance: &NodeInstance,
        results: &mut HashMap<NodeInstanceId, InstanceArgs>,
    ) -> eyre::Result<()> {
        // ensure all nodes this instance depends on are updated
        for connection in instance.input_connections.values() {
            if results.contains_key(&connection.instance) {
                continue;
            };

            let instance = self.get_instance(connection.instance)?;
            Box::pin(self.update_node_recursive(instance, results)).await?;
        }

        // collect args
        let mut args = InstanceRefArgs::default();

        for (arg_name, connection) in &instance.input_connections {
            let input_instance_result = results
                .get(&connection.instance)
                .context("Result not found")?;

            let arg_value = input_instance_result
                .get(arg_name)
                .context("Argument not found in the instance")?;

            args.insert(arg_name, arg_value);
        }

        // update node
        let node = self.get_node(&instance.node_id)?;
        let update_result = node.run(instance, self, &args).await?;

        // cache result for nodes depending on the current node instance
        results.insert(instance.instance_id, update_result);

        Ok(())
    }

    pub fn is_all_nodes_connected(&self) -> eyre::Result<bool> {
        for instance in self.instances.values() {
            if !self.is_node_connected(instance)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn is_node_connected(&self, instance: &NodeInstance) -> eyre::Result<bool> {
        let node = self.get_node(&instance.node_id)?;
        if node.is_root() {
            return Ok(true);
        }

        for (arg_name, arg_ty) in node.input_args() {
            if arg_ty.is_optional {
                // Optional arguments are not required to be connected
                continue;
            }

            if instance.input_connections.get(arg_name).is_none() {
                return Ok(false);
            };
        }

        Ok(true)
    }

    pub fn get_root_nodes(&self) -> eyre::Result<Vec<&NodeInstance>> {
        let mut root_nodes = Vec::new();

        for instance in self.instances.values() {
            let node = self.get_node(&instance.node_id)?;
            if node.is_root() {
                root_nodes.push(instance);
            }
        }

        Ok(root_nodes)
    }

    pub fn get_leaf_nodes(&self) -> Vec<&NodeInstance> {
        self.instances
            .values()
            .filter(|instance| instance.is_leaf())
            .collect::<Vec<_>>()
    }
}
