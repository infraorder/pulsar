use crate::util::{MANTLE, MAROON};
use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec2,
    render::color::Color,
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

pub trait NodeTrait {
    fn name(&self) -> String;
    fn display(&self) -> String;
    fn pos(&self) -> Position;
    fn get_active(&self) -> ColorPair;
    fn get_inert(&self) -> ColorPair;
    fn get_inactive(&self) -> ColorPair;
}

// TODO: use this
pub trait ParentNode {
    fn get_data(&self) -> &NodeData;
    fn get_node(&self) -> &Node;

    fn get_data_mut(&mut self) -> &mut NodeData;
    fn get_node_mut(&mut self) -> &mut Node;
}

// components
/// This Component denotes a node that is not part of a grid.
/// Instead this is a node blueprint for creating nodes of this type.
#[derive(Component, Clone)]
pub struct NodeBP;

#[derive(Component, Clone, Default)]
pub struct Pulse {
    pub direction: Position,
}

/// This Component denotes that a node is not setup yet
#[derive(Component, Clone)]
pub struct NotSetup;

/// All entities with this node deal with audio processing.
#[derive(Component, Clone)]
pub struct AudioNode {
    pub connection: Option<Entity>,
}

#[derive(Debug, Clone, Component)]
pub struct OutputSlot {
    pub idx: usize,
    pub slot: SlotNode,
}

impl NodeTrait for OutputSlot {
    fn name(&self) -> String {
        self.slot.name.clone()
    }

    fn display(&self) -> String {
        self.slot.display.clone()
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

impl NodeTrait for InputSlot {
    fn name(&self) -> String {
        self.slot.name.clone()
    }

    fn display(&self) -> String {
        self.slot.display.clone()
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

    pub fn to_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn offset(&self, pos: Position) -> Position {
        Position::new(self.x + pos.x, self.y + pos.y)
    }
}

// Slot type
/// This is a slot on a node. Sits close to the parent node [node][slot]
#[derive(Clone, Debug, Default)]
pub struct Slot {
    pub pos: Position,
    pub slot_type: SlotType,
    pub signal_type: NodeType,
    pub direction: Position,
}

#[derive(Component, Clone, Debug)]
pub struct SlotNode {
    pub slot_type: SlotType,
    pub signal_type: NodeType,
    pub pos: Position,
    pub display: String,
    pub name: String,
    pub active: ColorPair,
    pub inert: ColorPair,
    pub inactive: ColorPair,
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

    pub ntype: Vec<NodeType>,
    pub slots: Vec<Slot>,
    pub output_slots: Vec<Slot>,
}

/// Data object for the node - All nodes should have this struct.
#[derive(Clone, Default, Debug)]
pub struct NodeData {
    pub data: SlotData,
    pub slot_data: Vec<SlotData>,
    pub output_slot_data: Vec<SlotData>,

    pub updated: Vec<Position>,
    pub state: NodeStatus,

    pub commands: Vec<NodeCommand>,
}

#[derive(Clone, Default, Debug)]
pub enum NodeCommand {
    SpawnNode(Node),
    #[default]
    None,
}

// TYPES
/// Current state of the node - in order to control logic
#[derive(Clone, Default, Debug)]
pub enum NodeStatus {
    Active,
    Inert,
    #[default]
    Inactive,
    None,
}

/// Type of a node, in order to control logic
#[derive(Clone, Debug, Default)]
pub enum NodeType {
    Signal,
    SignalConst,
    SignalLink, // realtime signal for audio

    Prod,

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
#[derive(Clone, Debug, Default)]
pub enum SlotData {
    F32(f32),
    I32(i32),

    F32x2((f32, f32)),

    Bang(bool),
    #[default]
    None,
}

#[derive(Clone, Debug)]
pub enum ChannelType {
    Instrument,
    Transmitter,
}
