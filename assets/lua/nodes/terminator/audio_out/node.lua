node.display = "O"
node.name = "audio_out"
node.type = { NODE_TYPES.Receiver, NODE_TYPES.SignalConst }
node.slots = {
    {
        signal_type = NODE_TYPES.SignalConst,
        slot_type = SLOT_TYPE.F32x2,
        pos = { x = 0, y = 1 },
        direction = { x = 0, y = 0 },
    },
}
node.output_slots = {}
