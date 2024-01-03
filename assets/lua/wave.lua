FN = function(t, frequency, t_size)
    local f = 144.0 * TAU

    local lwave = W(math.sin, t, f, t_size)
    lwave = lwave * 0.5 + W(math.cos, t, f / 16.0, t_size) * 0.5
    lwave = lwave * (W(math.cos, t, f / 3.0, t_size) / 2.0)
    lwave = lwave + W(math.cos, t, f, t_size) * 0.5

    local rwave = W(math.cos, t, f, t_size)
    rwave = rwave * (W(math.sin, t, f / 3.0, t_size) / 2.0)

    return lwave, rwave
end
