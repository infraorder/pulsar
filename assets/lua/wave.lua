OUT0 = function(t, frequency, t_size)
    local wave = W(math.sin, t, frequency * 0.5, t_size);

    wave = (wave * 0.5) + (W(math.cos, t, 3.0, t_size) * 0.5);

    return wave
end

OUT1 = function(t, frequency, t_size)
    local wave = W(math.cos, t, frequency, t_size);

    wave = wave * W(math.sin, t, 3.0 * 0.75, t_size);

    wave = wave * 0.5 + (W(math.sin, t, 1.0, t_size) * 0.5);

    return wave
end
