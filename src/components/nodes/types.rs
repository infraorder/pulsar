// imports
use std::sync::Mutex;

use bevy::{asset::Handle, ecs::component::Component, render::color::Color, math::Vec2};
use rlua::{Context, Error, FromLua, Lua, Table};

use crate::{
    components::lua::LuaAsset,
    util::{MANTLE, MAROON},
};

// Color
/// Struct for passing colors to lua.
#[derive(Clone, Debug)]
pub struct PColor(pub Color);

/// Struct for passing Color Pairs to lua.
#[derive(Clone, Debug)]
pub struct ColorPair {
    pub foreground: PColor,
    pub background: PColor,
}

impl ColorPair {
    pub fn new(fg: Color, bg: Color) -> Self {
        Self {
            foreground: PColor(fg),
            background: PColor(bg),
        }
    }
}

impl Default for ColorPair {
    fn default() -> Self {
        Self {
            foreground: PColor(MANTLE),
            background: PColor(MAROON),
        }
    }
}

pub trait Display {
    fn name(&self) -> char;
    fn pos(&self) -> Position;
    fn get_active(&self) -> ColorPair;
    fn get_inert(&self) -> ColorPair;
    fn get_inactive(&self) -> ColorPair;
}

// components
/// This Component denotes a node that is not part of a grid.
/// Instead this is a node blueprint for creating nodes of this type.
#[derive(Component, Clone)]
pub struct NodeBP;

/// This Component denotes that a node is not setup yet
#[derive(Component, Clone)]
pub struct NotSetup;

/// All entities with this node deal with audio processing.
#[derive(Component, Clone)]
pub struct AudioNode;

// Position
/// x, y position on the grid.
#[derive(Clone, Debug, Default)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

// Slot type
/// This is a slot on a node. Sits close to the parent node [node][slot]
#[derive(Clone, Debug, Default)]
pub struct Slot {
    pub pos: Position,
    pub slot_type: SlotType,
    pub signal_type: ChainType,
}

// Lua
/// LuaNode - lua controlled node.
#[derive(Component)]
pub struct LuaNode {
    pub handles: Vec<LuaHandle>,
    pub node: Node,
    pub data: NodeData,
    pub lua: Mutex<Lua>,
}

impl Display for LuaNode {
    fn name(&self) -> char {
        self.node.name.chars().next().unwrap()
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

/// LuaHandle - handle for lua scripts.
#[derive(Clone)]
pub struct LuaHandle {
    pub ltype: LuaType,
    pub handle: Handle<LuaAsset>,
}

/// LuaType - type of lua script.
#[derive(Clone)]
pub enum LuaType {
    Wave,
    Node,
}

// Node
/// Node - base node. All nodes should have this struct.
#[derive(Clone, Default, Debug)]
pub struct Node {
    pub name: String,
    pub display: String,
    pub pos: Position,
    pub active: ColorPair,
    pub inert: ColorPair,
    pub inactive: ColorPair,
    pub ntype: Vec<ChainType>,
    pub slots: Vec<Slot>,
    pub output_slots: Vec<Slot>,
}

/// Data object for the node - All nodes should have this struct.
#[derive(Clone, Default, Debug)]
pub struct NodeData {
    pub slot_data: Vec<SlotE>,
    pub output_slot_data: Vec<SlotE>,
    pub updated: Vec<Position>,
    pub state: NodeState,
}

// TYPES
/// Current state of the node - in order to control logic
#[derive(Clone, Default, Debug)]
pub enum NodeState {
    Active,
    Inert,
    #[default]
    Inactive,
    None,
}

/// Type of a node, in order to control logic
#[derive(Clone, Debug, Default)]
pub enum ChainType {
    Signal,
    SignalConst,
    SignalLink, // realtime signal for audio
    Emitter,
    Receiver,
    #[default]
    None, // Default node type -- will throw if this is type
}

// Slot Type
#[derive(Clone, Debug, Default)]
pub enum SlotType {
    F32,
    I32,

    F32x2,
    I32x2,

    Bang,
    #[default]
    None, // default slot type -- will throw if this is type
}

// Slot type - for converting from lua
#[derive(Clone, Debug)]
pub enum SlotE {
    F32(f32),
    I32(i32),

    F32x2((f32, f32)),

    Bang(bool),
}

// Impl of FromLua for all types
impl<'lua> FromLua<'lua> for Node {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut node = Node::default();
                node.name = table.get::<_, String>("name")?;
                node.display = table.get::<_, String>("display")?;

                node.pos = table.get::<_, Position>("pos")?;

                node.active = table.get::<_, ColorPair>("active")?;
                node.inert = table.get::<_, ColorPair>("inert")?;
                node.inactive = table.get::<_, ColorPair>("inactive")?;

                node.ntype = table
                    .get::<_, Table>("ntype")?
                    .sequence_values::<ChainType>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.slots = table
                    .get::<_, Table>("slots")?
                    .sequence_values::<Slot>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.output_slots = table
                    .get::<_, Table>("output_slots")?
                    .sequence_values::<Slot>()
                    .map(|tv| tv.unwrap())
                    .collect();

                Ok(node)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "Node",
                to: "Node",
                message: Some("Table node does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for ColorPair {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut node = ColorPair::default();

                node.foreground = table.get::<_, PColor>("foreground")?;
                node.background = table.get::<_, PColor>("background")?;

                Ok(node)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "ColorPair",
                to: "Table",
                message: Some("ColorPair does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for PColor {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut color = Color::default();

                color.set_r(table.get::<_, f32>(1)?);
                color.set_g(table.get::<_, f32>(2)?);
                color.set_b(table.get::<_, f32>(3)?);

                Ok(PColor(color))
            }
            _ => Err(Error::FromLuaConversionError {
                from: "Color",
                to: "PColor",
                message: Some("Color Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for ChainType {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "Signal" => ChainType::Signal,
                "SignalConst" => ChainType::SignalConst,
                "SignalLink" => ChainType::SignalLink,
                "Emitter" => ChainType::Emitter,
                "Receiver" => ChainType::Receiver,
                _ => ChainType::None,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for Slot {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut slot = Slot::default();

                slot.pos = table.get::<_, Position>("pos")?;

                slot.signal_type = table.get::<_, ChainType>("signal_type")?;

                slot.slot_type = table.get::<_, SlotType>("slot_type")?;

                Ok(slot)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for SlotType {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "F32" => SlotType::F32,
                "I32" => SlotType::I32,

                "F32x2" => SlotType::F32x2,
                "I32x2" => SlotType::I32x2,

                "Bang" => SlotType::Bang,

                _ => SlotType::None,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "SlotType",
                message: Some("SlotType Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for NodeData {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut node = NodeData::default();

                node.slot_data = table
                    .get::<_, Table>("slot_data")?
                    .sequence_values::<SlotE>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.output_slot_data = table
                    .get::<_, Table>("output_slot_data")?
                    .sequence_values::<SlotE>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.updated = table
                    .get::<_, Table>("updated")?
                    .sequence_values::<Position>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.state = table.get::<_, NodeState>("state")?;

                Ok(node)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "Node",
                to: "Node",
                message: Some("Table node does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for SlotE {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Integer(i) => Ok(SlotE::I32(i as i32)),
            rlua::Value::Number(i) => Ok(SlotE::F32(i as f32)),
            rlua::Value::Boolean(i) => Ok(SlotE::Bang(i)),
            rlua::Value::Table(t) => Ok(SlotE::F32x2((t.get::<_, f32>(1)?, t.get::<_, f32>(2)?))),
            _ => Err(Error::FromLuaConversionError {
                from: "Node",
                to: "Node",
                message: Some("Table node does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for Position {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut pos = Position::default();

                pos.x = table.get::<_, i32>("x")?;
                pos.y = table.get::<_, i32>("y")?;

                Ok(pos)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for NodeState {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "Active" => NodeState::Active,
                "Inactive" => NodeState::Inactive,
                "Inert" => NodeState::Inert,
                _ => NodeState::None,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}
