use bevy::ecs::component::Component;

use super::types::{ColorPair, Node, NodeData, NodeTrait, Position, ParentNode};

#[derive(Component)]
pub struct IsNativeNode;

#[derive(Clone, Debug)]
pub struct NativeNode {
    pub node: Node,
    pub data: NodeData,
}

impl NodeTrait for NativeNode {
    fn name(&self) -> String {
        self.node.name.clone()
    }

    fn display(&self) -> String {
        self.node.display.clone()
    }

    fn pos(&self) -> Position {
        self.node.pos.clone()
    }

    fn get_active(&self) -> ColorPair {
        self.node.active.clone()
    }

    fn get_inert(&self) -> ColorPair {
        self.node.inert.clone()
    }

    fn get_inactive(&self) -> ColorPair {
        self.node.inactive.clone()
    }
}

impl ParentNode for NativeNode {
    fn get_node(&self) -> &Node {
        &self.node
    }

    fn get_data(&self) -> &NodeData {
        &self.data
    }

    fn get_node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    fn get_data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }
}
