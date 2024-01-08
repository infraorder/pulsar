pub mod from_lua;

use std::sync::Mutex;

use bevy::{
    asset::{Assets, Handle},
    ecs::{component::Component, system::Res},
};
use rlua::Lua;

use crate::components::lua::LuaAsset;

use super::types::{ColorPair, Node, NodeData, NodeTrait, ParentNode, Position, NodeVarient};

use crate::lua::load_fn;

pub fn init_lua<T: ParentNode>(lua_assets: &Res<Assets<LuaAsset>>, node: &mut T) {
    if let (Some(handles), Some(lua)) = (node.get_lua_handles(), node.get_lua()) {
        let lua = lua.lock().unwrap();

        handles.iter().for_each(|handle| {
            if let LuaType::Node = handle.ltype {
                let lua_asset = lua_assets.get(handle.handle.clone()); // TODO: Handle unwrap

                match lua_asset {
                    Some(lua_asset) => {
                        load_fn(&lua, &node.get_node().name.to_string(), &lua_asset.script);
                    }
                    None => panic!("this should never happen"),
                }
            }
        });
        drop(lua);
    }
}

// Lua
/// LuaNode - lua controlled node.
#[derive(Debug)]
pub struct LuaNode {
    pub node: Node,
    pub data: NodeData,
    pub handles: Vec<LuaHandle>,
    pub lua: Mutex<Lua>,
}

impl NodeTrait for LuaNode {
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

impl ParentNode for LuaNode {
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
        Some(&self.lua)
    }

    fn get_lua_mut(&mut self) -> Option<&mut Mutex<Lua>> {
        Some(&mut self.lua)
    }

    fn get_lua_handles(&self) -> Option<&Vec<LuaHandle>> {
        Some(&self.handles)
    }

    fn get_lua_handles_mut(&mut self) -> Option<&mut Vec<LuaHandle>> {
        Some(&mut self.handles)
    }
}

#[derive(Component)]
pub struct IsLuaNode;

/// LuaHandle - handle for lua scripts.
#[derive(Clone, Debug)]
pub struct LuaHandle {
    pub ltype: LuaType,
    pub handle: Handle<LuaAsset>,
}

/// LuaType - type of lua script.
#[derive(Clone, Debug)]
pub enum LuaType {
    Wave,
    Node,
}
