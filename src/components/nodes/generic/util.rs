use crate::components::nodes::{
    native::NativeNode,
    types::{Node, NodeData, NodeStatus, NodeType, SlotData, Position},
};

use super::GenericNode;

pub fn construct_pulse_node(
    pos: Position,
    name: String,
    display: String,
    ntype: Vec<NodeType>,
    data: SlotData,
) -> GenericNode {
    GenericNode::Native(NativeNode {
        data: NodeData {
            // data: node.get_node().output_slots[i],
            data,
            state: NodeStatus::Active,
            ..Default::default()
        },
        node: Node {
            name,
            display,
            ntype,
            pos,
            ..Default::default()
        },
    })
}
