use std::sync::Mutex;

use bevy::ecs::component::Component;
use rlua::Lua;

use super::{
    lua::LuaHandle,
    types::{ColorPair, Node, NodeData, NodeTrait, ParentNode, Position, NodeVarient},
};

#[derive(Component)]
pub struct IsNativeNode;

#[derive(Debug)]
pub struct NativeNode {
    pub node: Node,
    pub data: NodeData,
    pub handles: Option<Vec<LuaHandle>>,
    pub lua: Option<Mutex<Lua>>,
}

impl NodeTrait for NativeNode {
    fn name(&self) -> NodeVarient {
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

    fn get_lua(&self) -> Option<&Mutex<Lua>> {
        self.lua.as_ref()
    }

    fn get_lua_mut(&mut self) -> Option<&mut Mutex<Lua>> {
        self.lua.as_mut()
    }

    fn get_lua_handles(&self) -> Option<&Vec<LuaHandle>> {
        self.handles.as_ref()
    }

    fn get_lua_handles_mut(&mut self) -> Option<&mut Vec<LuaHandle>> {
        self.handles.as_mut()
    }
}
