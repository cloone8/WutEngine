---@meta
---stubs for the native WutEngine log module

--- A four-component (x, y, z, w) vector
---@class Vec4
---@field x number The first component
---@field y number The second component
---@field z number The third component
---@field w number The fourth component
---@operator add(Vec4): Vec4
---@operator add(number): Vec4
---@operator sub(Vec4): Vec4
---@operator sub(number): Vec4
local Vec4 = {}

local M = {}

--- Creates a new Vec4 with all-zero components
--- @return Vec4
function M.vec4() end

--- Creates a new Vec4 from named components
--- @param components {x?: number, y?: number, z?: number, w?: number}
--- @return Vec4
function M.vec4(components) end

--- Creates a new Vec4 with the given `x`, with `y`, `z`, `w` set to 0
--- @param x number
--- @return Vec4
function M.vec4(x) end

--- Creates a new Vec4 with the given `x` and `y`, with `z`, `w` set to 0
--- @param x number
--- @param y number
--- @return Vec4
function M.vec4(x, y) end

--- Creates a new Vec4 with the given `x`, `y` and `z`, with `w` set to 0
--- @param x number
--- @param y number
--- @param z number
--- @return Vec4
function M.vec4(x, y, z) end

--- Creates a new Vec4 with the given `x`, `y`, `z` and `w`
--- @param x number
--- @param y number
--- @param z number
--- @param w number
--- @return Vec4
function M.vec4(x, y, z, w) end

--- Creates a new Vec4 by setting `x`, `y`, `z`, and `w` to `n`
---@param n number
---@return Vec4
function M.splat4(n) end

return M
