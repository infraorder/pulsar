// imports
use std::sync::Mutex;

use bevy::{asset::Handle, ecs::component::Component, math::Vec2, render::color::Color};
use rlua::{Context, Error, FromLua, Lua, Table, ToLua};

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

#[derive(Debug, Clone, Component)]
pub struct OutputSlot {
    pub idx: usize,
    pub slot: SlotNode,
}

impl Display for OutputSlot {
    fn name(&self) -> char {
        self.slot.display.chars().next().unwrap()
    }

    fn pos(&self) -> Position {
        self.slot.pos.clone()
    }

    fn get_active(&self) -> ColorPair {
        self.slot.active.clone()
    }

    fn get_inert(&self) -> ColorPair {
        self.slot.inert.clone()
    }

    fn get_inactive(&self) -> ColorPair {
        self.slot.inactive.clone()
    }
}

#[derive(Debug, Clone, Component)]
pub struct InputSlot {
    pub idx: usize,
    pub slot: SlotNode,
}

impl Display for InputSlot {
    fn name(&self) -> char {
        self.slot.display.chars().next().unwrap()
    }

    fn pos(&self) -> Position {
        self.slot.pos.clone()
    }

    fn get_active(&self) -> ColorPair {
        self.slot.active.clone()
    }

    fn get_inert(&self) -> ColorPair {
        self.slot.inert.clone()
    }

    fn get_inactive(&self) -> ColorPair {
        self.slot.inactive.clone()
    }
}

// Position
/// x, y position on the grid.
#[derive(Clone, Debug, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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

#[derive(Component, Clone, Debug)]
pub struct SlotNode {
    pub slot_type: SlotType,
    pub signal_type: ChainType,
    pub pos: Position,
    pub display: String,
    pub active: ColorPair,
    pub inert: ColorPair,
    pub inactive: ColorPair,
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

impl<'lua> ToLua<'lua> for Node {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        table.set("name", self.name)?;
        table.set("display", self.display)?;
        table.set("pos", self.pos)?;
        table.set("active", self.active)?;
        table.set("inert", self.inert)?;
        table.set("inactive", self.inactive)?;
        let ntype = ctx.create_table()?;
        for (i, t) in self.ntype.iter().enumerate() {
            ntype.set(i + 1, t.clone())?;
        }
        table.set("ntype", ntype)?;
        let slots = ctx.create_table()?;
        for (i, t) in self.slots.iter().enumerate() {
            slots.set(i + 1, t.clone())?;
        }
        table.set("slots", slots)?;
        let output_slots = ctx.create_table()?;
        for (i, t) in self.output_slots.iter().enumerate() {
            output_slots.set(i + 1, t.clone())?;
        }
        table.set("output_slots", output_slots)?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for ColorPair {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        table.set("foreground", self.foreground)?;
        table.set("background", self.background)?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for PColor {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        table.set(1, self.0.r())?;
        table.set(2, self.0.g())?;
        table.set(3, self.0.b())?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for Slot {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        table.set("pos", self.pos)?;
        table.set("signal_type", self.signal_type)?;
        table.set("slot_type", self.slot_type)?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for ChainType {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            ChainType::Signal => Ok("Signal".to_lua(ctx)?),
            ChainType::SignalConst => Ok("SignalConst".to_lua(ctx)?),
            ChainType::SignalLink => Ok("SignalLink".to_lua(ctx)?),
            ChainType::Emitter => Ok("Emitter".to_lua(ctx)?),
            ChainType::Receiver => Ok("Receiver".to_lua(ctx)?),
            ChainType::None => Ok("None".to_lua(ctx)?),
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

impl<'lua> ToLua<'lua> for SlotType {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            SlotType::F32 => Ok("F32".to_lua(ctx)?),
            SlotType::I32 => Ok("I32".to_lua(ctx)?),
            SlotType::F32x2 => Ok("F32x2".to_lua(ctx)?),
            SlotType::I32x2 => Ok("I32x2".to_lua(ctx)?),
            SlotType::Bang => Ok("Bang".to_lua(ctx)?),
            SlotType::None => Err(Error::ToLuaConversionError {
                from: "SlotType",
                to: "String",
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

impl<'lua> ToLua<'lua> for NodeData {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        let slot_data = ctx.create_table()?;
        for (i, slot) in self.slot_data.iter().enumerate() {
            slot_data.set(i + 1, slot.clone().to_lua(ctx)?)?;
        }
        table.set("slot_data", slot_data)?;
        let output_slot_data = ctx.create_table()?;
        for (i, slot) in self.output_slot_data.iter().enumerate() {
            output_slot_data.set(i + 1, slot.clone().to_lua(ctx)?)?;
        }
        table.set("output_slot_data", output_slot_data)?;
        let updated = ctx.create_table()?;
        for (i, pos) in self.updated.iter().enumerate() {
            updated.set(i + 1, pos.clone().to_lua(ctx)?)?;
        }
        table.set("updated", updated)?;
        table.set("state", self.state.to_lua(ctx)?)?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for SlotE {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            SlotE::F32(f) => Ok(f.to_lua(ctx)?),
            SlotE::I32(i) => Ok(i.to_lua(ctx)?),
            SlotE::Bang(b) => Ok(b.to_lua(ctx)?),
            SlotE::F32x2(t) => {
                let table = ctx.create_table()?;
                table.set(1, t.0)?;
                table.set(2, t.1)?;

                Ok(table.to_lua(ctx)?)
            }
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

impl<'lua> ToLua<'lua> for Position {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        let table = ctx.create_table()?;
        table.set("x", self.x)?;
        table.set("y", self.y)?;
        Ok(table.to_lua(ctx)?)
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

impl<'lua> ToLua<'lua> for NodeState {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            NodeState::Active => Ok("Active".to_lua(ctx)?),
            NodeState::Inactive => Ok("Inactive".to_lua(ctx)?),
            NodeState::Inert => Ok("Inert".to_lua(ctx)?),
            _ => Ok("None".to_lua(ctx)?),
        }
    }
}
