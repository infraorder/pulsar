node.display = "R"
node.name = "lua_read"
node.type = { NODE_TYPES.Receiver, NODE_TYPES.SignalConst }
node.slots = {
    { signal_type = NODE_TYPES.SignalConst, slot_type = SLOT_TYPE.F32x2, pos = { x = 0, y = 1 } },
}
node.output_slots = {
    { signal_type = NODE_TYPES.SignalLink, slot_type = SLOT_TYPE.F32x2, pos = { x = 0, y = -1 } },
}
