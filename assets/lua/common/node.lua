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
    pos = { x = 0, y = 0 },
    display = "X",
    name = "X",
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
    ntype = { NODE_TYPES.None },
    slots = {},
    output_slots = {},
}

data = {
    updated = {},
    slot_data = {},
    output_slot_data = {},
    state = NODE_STATE.None,
    data = nil,
}

RESET = function()
    data.updated = {}
    data.slot_data = {}
    data.output_slot_data = {}
    data.state = NODE_STATE.None
end
