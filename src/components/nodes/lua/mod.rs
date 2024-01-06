pub mod from_lua;

use std::sync::Mutex;

use bevy::{
    asset::{Assets, Handle},
    ecs::{component::Component, system::Res},
};
use rlua::Lua;

use crate::components::lua::LuaAsset;

use super::types::{ColorPair, Node, NodeData, NodeTrait, ParentNode, Position};

use crate::lua::load_fn;

pub fn init_lua(lua_assets: &Res<Assets<LuaAsset>>, node: &mut LuaNode) {
    let lua = node.lua.lock().unwrap();
    node.handles.iter().for_each(|handle| {
        if let LuaType::Node = handle.ltype {
            let lua_asset = lua_assets.get(handle.handle.clone()); // TODO: Handle unwrap

            match lua_asset {
                Some(lua_asset) => {
                    load_fn(&lua, &node.node.name, &lua_asset.script);
                }
                None => panic!("this should never happen"),
            }
        }
    });
    let n = node.node.clone();
    let nd = node.data.clone();

    node.node = n;
    node.data = nd;
}

// Lua
/// LuaNode - lua controlled node.
#[derive(Debug)]
pub struct LuaNode {
    pub handles: Vec<LuaHandle>,
    pub node: Node,
    pub data: NodeData,
    pub lua: Mutex<Lua>,
}

impl NodeTrait for LuaNode {
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
