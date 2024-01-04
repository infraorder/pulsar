TAU = math.pi * 2.0

P = {}
W_I = 0

T = 0.0
TS = 0.0

OUT_FN = function(t, frequency, t_size)
    W_I = 0
    T = t
    TS = t_size

    return FN(frequency)
end

GP = function(f)
    local current_phase = P[W_I]

    if current_phase == nil then
        P[W_I] = 0.0
        current_phase = 0.0
    end

    P[W_I] = (current_phase + (TS * f)) % TAU

    return current_phase
end

W = function(wave_fn, f)
    W_I = W_I + 1

    local phase = GP(f)
    local res = wave_fn(T * f + phase)
    return res
end
