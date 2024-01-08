use crate::components::nodes::{
    native::NativeNode,
    types::{
        Node, NodeData, NodeStatus, NodeTrait, NodeType, NodeVarient, ParentNode, Position, Pulse,
        SlotData,
    },
};

use super::GenericNode;

pub fn construct_pulse_node(
    pos: Position,
    name: NodeVarient,
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
        lua: None,
        handles: None,
    })
}

pub fn calculate_grid_pos<T: ParentNode>(node: &T, pulse: &Pulse, pos: Position) -> Position {
    pos
        .offset(&node.get_node().pos)
        .offset(&node.get_node().output_slots[pulse.slot_idx].pos)
}
