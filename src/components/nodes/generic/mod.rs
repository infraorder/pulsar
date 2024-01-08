pub mod system;
pub mod types;
pub mod util;

use std::sync::Mutex;

use bevy::ecs::component::Component;
use rlua::Lua;

use super::{
    lua::{LuaHandle, LuaNode},
    native::NativeNode,
    types::{ColorPair, Node, NodeData, NodeTrait, NodeVarient, ParentNode, Position},
};

#[derive(Component, Debug)]
pub enum GenericNode {
    Lua(LuaNode),
    Native(NativeNode),
}

impl GenericNode {
    pub fn get_lua_node(&self) -> Option<&LuaNode> {
        match self {
            GenericNode::Lua(node) => Some(node),
            _ => None,
        }
    }

    pub fn get_native_node(&self) -> Option<&NativeNode> {
        match self {
            GenericNode::Native(node) => Some(node),
            _ => None,
        }
    }

    pub fn get_lua_node_mut(&mut self) -> Option<&mut LuaNode> {
        match self {
            GenericNode::Lua(node) => Some(node),
            _ => None,
        }
    }

    pub fn get_native_node_mut(&mut self) -> Option<&mut NativeNode> {
        match self {
            GenericNode::Native(node) => Some(node),
            _ => None,
        }
    }
}

impl NodeTrait for GenericNode {
    fn name(&self) -> NodeVarient {
        match self {
            GenericNode::Lua(node) => node.name(),
            GenericNode::Native(node) => node.name(),
        }
    }

    fn display(&self) -> String {
        match self {
            GenericNode::Lua(node) => node.display(),
            GenericNode::Native(node) => node.display(),
        }
    }

    fn pos(&self) -> Position {
        match self {
            GenericNode::Lua(node) => node.pos(),
            GenericNode::Native(node) => node.pos(),
        }
    }

    fn get_active(&self) -> ColorPair {
        match self {
            GenericNode::Lua(node) => node.get_active(),
            GenericNode::Native(node) => node.get_active(),
        }
    }

    fn get_inert(&self) -> ColorPair {
        match self {
            GenericNode::Lua(node) => node.get_inert(),
            GenericNode::Native(node) => node.get_inert(),
        }
    }

    fn get_inactive(&self) -> ColorPair {
        match self {
            GenericNode::Lua(node) => node.get_inactive(),
            GenericNode::Native(node) => node.get_inactive(),
        }
    }
}

impl ParentNode for GenericNode {
    fn get_node(&self) -> &Node {
        match self {
            GenericNode::Lua(node) => node.get_node(),
            GenericNode::Native(node) => node.get_node(),
        }
    }
    fn get_data(&self) -> &NodeData {
        match self {
            GenericNode::Lua(node) => node.get_data(),
            GenericNode::Native(node) => node.get_data(),
        }
    }

    fn get_node_mut(&mut self) -> &mut Node {
        match self {
            GenericNode::Lua(node) => node.get_node_mut(),
            GenericNode::Native(node) => node.get_node_mut(),
        }
    }
    fn get_data_mut(&mut self) -> &mut NodeData {
        match self {
            GenericNode::Lua(node) => node.get_data_mut(),
            GenericNode::Native(node) => node.get_data_mut(),
        }
    }

    fn get_lua(&self) -> Option<&Mutex<Lua>> {
        match self {
            GenericNode::Lua(node) => node.get_lua(),
            GenericNode::Native(node) => node.get_lua(),
        }
    }

    fn get_lua_mut(&mut self) -> Option<&mut Mutex<Lua>> {
        match self {
            GenericNode::Lua(node) => node.get_lua_mut(),
            GenericNode::Native(node) => node.get_lua_mut(),
        }
    }

    fn get_lua_handles(&self) -> Option<&Vec<LuaHandle>> {
        match self {
            GenericNode::Lua(node) => node.get_lua_handles(),
            GenericNode::Native(node) => node.get_lua_handles(),
        }
    }

    fn get_lua_handles_mut(&mut self) -> Option<&mut Vec<LuaHandle>> {
        match self {
            GenericNode::Lua(node) => node.get_lua_handles_mut(),
            GenericNode::Native(node) => node.get_lua_handles_mut(),
        }
    }
}
