NODE_TYPES = {
    Signal = "Signal",
    SignalConst = "SignalConst",
    SignalLink = "SignalLink",
    Emitter = "Emitter",
    Receiver = "Receiver",
    None = "None", -- default node_type required to be changed
}

SLOT_TYPE = {
    F32 = "F32",
    I32 = "I32",

    F32x2 = "F32x2",

    Bang = "Bang",

    None = "None",
}

NODE_STATE = {
    Active = "Active",
    Inactive = "Inactive",
    Inert = "Inert",
    None = "None",
}

node = {
    inert = {
        foreground = TEXT,
        background = SURFACE2,
    },
    active = {
        foreground = BASE,
        background = RED,
    },
    inactive = {
        foreground = BASE,
        background = MANTLE,
    },
    pos = { x = 0, y = 0 },
    display = "X",
    name = "X",
    ntype = { NODE_TYPES.None },
    slots = {},
    output_slots = {},
}

data = {
    updated = {},
    slot_data = {},
    output_slot_data = {},
    state = NODE_STATE.None,
}

RESET = function()
    node_data.updated = {}
    node_data.slot_data = {}
    node_data.output_slot_data = {}
    node_data.state = NODE_STATE.None
end
