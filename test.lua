package.cpath = package.cpath .. ';' .. os.getenv('HOME') .. '/RustroverProjects/tomato/target/debug/?.so'
local tomato = require("libtomato").new(5)

local wall_start = os.time()
local cpu_start = os.clock()

print("Starting timer...")
tomato:start()

local last = tomato:get_remaining()

while tomato:get_remaining() > 0 do
    local current = tomato:get_remaining()
    if current ~= last then
        print(string.format("Remaining: %d (Δ %.3f s)", current, os.clock() - cpu_start))
        last = current
    end
end

print(string.format("\nTimer finished! (CPU: %.3f s, Wall: %d s)",
        os.clock() - cpu_start, os.time() - wall_start))