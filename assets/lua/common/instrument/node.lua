node.active.foreground = LAVENDER
node.type = { NODE_TYPES.Receiver }
node.slots = {
    { signal_type = NODE_TYPES.Signal, slot_type = SLOT_TYPE.Bang, pos = { x = -1, y = 0 } },
}
node.output_slots = {
    { signal_type = NODE_TYPES.SignalLink, slot_type = SLOT_TYPE.F32x2, pos = { x = 0, y = -1 } },
}
