TAU = math.pi * 2.0;

P = {};
W_I = 0;

OUT_FN = function(t, frequency, t_size)
    W_I = 0;
    return FN(t, frequency, t_size)
end

GP = function()
    local p = P[W_I];

    if p == nil then
        P[W_I] = 0.0;
        return 0.0;
    end

    return p;
end

UP = function(f, t_size)
    local p = P[W_I];

    -- self.phase = (self.phase + t * self.frequency.load(Ordering::Relaxed)) % TAU;
    P[W_I] = (p + (t_size * f)) % TAU;
end

W = function(wave_fn, t, f, t_size)
    W_I = W_I + 1;

    local phase = GP();
    local res = wave_fn(t * f + phase);
    UP(f, t_size);
    return res;
end

function D(o)
    if type(o) == 'table' then
        local s = '{ '
        for k, v in pairs(o) do
            if type(k) ~= 'number' then k = '"' .. k .. '"' end
            s = s .. '[' .. k .. '] = ' .. D(v) .. ','
        end
        return s .. '} '
    else
        return tostring(o)
    end
end
