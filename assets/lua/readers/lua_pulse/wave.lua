FN = function(_) -- frequency
    local f = 144.0 * TAU

    local lwave = W(math.sin, f)
    lwave = lwave * 0.5 + W(math.cos, f / 16.0) * 0.5
    lwave = lwave * (W(math.cos, f / 3.0) / 2.0)
    lwave = lwave + W(math.cos, f) * 0.5

    local rwave = W(math.cos, f)
    rwave = rwave * (W(math.sin, f / 3.0) / 2.0)

    return lwave, rwave
end
