FN = function(t, frequency, t_size)
    local f = 120.0 * TAU;

    local lwave = W(math.sin, t, f, t_size);
    lwave = lwave * (W(math.cos, t, f / 2.0, t_size) / 2.0);

    local rwave = W(math.cos, t, f, t_size);
    rwave = rwave * (W(math.sin, t, f / 3.0, t_size) / 2.0);

    return lwave, rwave
end
