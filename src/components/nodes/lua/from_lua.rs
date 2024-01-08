use bevy::render::color::Color;
use rlua::{Context, Error, FromLua, Table, ToLua};

use crate::components::nodes::types::NodeVarient;

use super::super::types::{
    ColorPair, Node, NodeData, NodeStatus, NodeType, PColor, Position, Slot, SlotData, SlotType,
};

// Impl of FromLua for all types
impl<'lua> FromLua<'lua> for Node {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Table(table) => {
                let mut node = Node::default();
                node.name = table.get::<_, NodeVarient>("name")?;
                node.display = table.get::<_, String>("display")?;

                node.pos = table.get::<_, Position>("pos")?;

                node.active = table.get::<_, ColorPair>("active")?;
                node.inert = table.get::<_, ColorPair>("inert")?;
                node.inactive = table.get::<_, ColorPair>("inactive")?;

                node.ntype = table
                    .get::<_, Table>("ntype")?
                    .sequence_values::<NodeType>()
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

                slot.signal_type = table.get::<_, NodeType>("signal_type")?;

                slot.slot_type = table.get::<_, SlotType>("slot_type")?;

                slot.direction = table.get::<_, Position>("direction")?;

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

impl<'lua> FromLua<'lua> for NodeType {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "Signal" => NodeType::Signal,
                "SignalConst" => NodeType::SignalConst,
                "SignalLink" => NodeType::SignalLink,
                "Emitter" => NodeType::Emitter,
                "Receiver" => NodeType::Receiver,
                _ => NodeType::None,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for NodeType {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            NodeType::Signal => Ok("Signal".to_lua(ctx)?),
            NodeType::SignalConst => Ok("SignalConst".to_lua(ctx)?),
            NodeType::SignalLink => Ok("SignalLink".to_lua(ctx)?),
            NodeType::Emitter => Ok("Emitter".to_lua(ctx)?),
            NodeType::Receiver => Ok("Receiver".to_lua(ctx)?),
            NodeType::Prod => Ok("Prod".to_lua(ctx)?),
            NodeType::None => Ok("None".to_lua(ctx)?),
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
                    .sequence_values::<SlotData>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.output_slot_data = table
                    .get::<_, Table>("output_slot_data")?
                    .sequence_values::<SlotData>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.updated = table
                    .get::<_, Table>("updated")?
                    .sequence_values::<usize>()
                    .map(|tv| tv.unwrap())
                    .collect();

                node.state = table.get::<_, NodeStatus>("state")?;

                node.data = table.get::<_, SlotData>("data")?;

                Ok(node)
            }
            _ => Err(Error::FromLuaConversionError {
                from: "NodeData",
                to: "NodeData",
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

impl<'lua> FromLua<'lua> for SlotData {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::Integer(i) => Ok(SlotData::I32(i as i32)),
            rlua::Value::Number(i) => Ok(SlotData::F32(i as f32)),
            rlua::Value::Boolean(i) => Ok(SlotData::Bang(i)),
            rlua::Value::Table(t) => {
                Ok(SlotData::F32x2((t.get::<_, f32>(1)?, t.get::<_, f32>(2)?)))
            }
            rlua::Value::Nil => Ok(SlotData::None),
            _ => Err(Error::FromLuaConversionError {
                from: "Node",
                to: "Node",
                message: Some("Table node does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for SlotData {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            SlotData::F32(f) => Ok(f.to_lua(ctx)?),
            SlotData::I32(i) => Ok(i.to_lua(ctx)?),
            SlotData::Bang(b) => Ok(b.to_lua(ctx)?),
            SlotData::F32x2(t) => {
                let table = ctx.create_table()?;
                table.set(1, t.0)?;
                table.set(2, t.1)?;

                Ok(table.to_lua(ctx)?)
            }
            SlotData::None => Ok(rlua::Value::Nil),
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

impl<'lua> FromLua<'lua> for NodeStatus {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "Active" => NodeStatus::Active,
                "Inactive" => NodeStatus::Inactive,
                "Inert" => NodeStatus::Inert,
                _ => NodeStatus::None,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for NodeStatus {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            NodeStatus::Active => Ok("Active".to_lua(ctx)?),
            NodeStatus::Inactive => Ok("Inactive".to_lua(ctx)?),
            NodeStatus::Inert => Ok("Inert".to_lua(ctx)?),
            _ => Ok("None".to_lua(ctx)?),
        }
    }
}

impl<'lua> FromLua<'lua> for NodeVarient {
    fn from_lua(value: rlua::prelude::LuaValue<'lua>, _ctx: Context<'lua>) -> Result<Self, Error> {
        match value {
            rlua::Value::String(str) => Ok(match str.to_str()? {
                "lua_read" => NodeVarient::LuaRead,
                "lua_pulse" => NodeVarient::LuaPulse,
                s => NodeVarient::Custom(s.to_string()),
            }),
            rlua::Value::Nil => Ok(NodeVarient::None),
            _ => Err(Error::FromLuaConversionError {
                from: "String",
                to: "ChainType",
                message: Some("Chain Type Does not exist".to_string()),
            }),
        }
    }
}

impl<'lua> ToLua<'lua> for NodeVarient {
    fn to_lua(self, ctx: Context<'lua>) -> Result<rlua::Value<'lua>, Error> {
        match self {
            NodeVarient::LuaRead => Ok("lua_read".to_lua(ctx)?),
            NodeVarient::LuaPulse => Ok("lua_pulse".to_lua(ctx)?),
            NodeVarient::Custom(s) => Ok(s.to_lua(ctx)?),
            NodeVarient::None => Ok(rlua::Value::Nil),
            _ => Ok("None".to_lua(ctx)?),
        }
    }
}
