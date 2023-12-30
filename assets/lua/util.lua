TAU = math.pi * 2.0;

P = {};
C_O = 0;
W_I = 0;

OUT_FN = function(t, frequency, t_size)
    return OUT0_FN(t, frequency, t_size), OUT1_FN(t, frequency, t_size)
end

OUT0_FN = function(t, frequency, t_size)
    C_O = 0;
    W_I = 0;

    return OUT0(t, frequency, t_size);
end

OUT1_FN = function(t, frequency, t_size)
    C_O = 1;
    W_I = 0;

    return OUT1(t, frequency, t_size);
end

GP = function()
    local p = P[C_O .. W_I];

    if p == nil then
        P[C_O .. W_I] = 0.0;
        return p;
    end

    return p;
end

UP = function(f, t_size)
    local p = P[C_O .. W_I];

    -- self.phase = (self.phase + t * self.frequency.load(Ordering::Relaxed)) % TAU;
    P[C_O .. W_I] = (p + (t_size * f)) % TAU;
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
