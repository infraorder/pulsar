FN = function(_) -- frequency
    local f = 80.0 * TAU

    local lwave = W(math.sin, f)
    lwave = (lwave * 0.5) + (W(math.cos, f / 1.0) * 0.5)
    lwave = lwave * W(math.cos, f / 3.01)

    local rwave = W(math.cos, f)
    rwave = rwave * W(math.sin, f / 1)
    local lfo = W(math.sin, 1.0)
    rwave = rwave * math.cos(lfo)

    return lwave, rwave
end
