---@meta
---stubs for the native WutEngine log module

local M = {}

---Logs the given arguments with level "trace". Arguments are concatenated using tabs, like `print()`
---@param ... any
function M.trace(...) end

---Logs the given arguments with level "debug". Arguments are concatenated using tabs, like `print()`
---@param ... any
function M.debug(...) end

---Logs the given arguments with level "info". Arguments are concatenated using tabs, like `print()`
---@param ... any
function M.info(...) end

---Logs the given arguments with level "warn". Arguments are concatenated using tabs, like `print()`
---@param ... any
function M.warn(...) end

---Logs the given arguments with level "error". Arguments are concatenated using tabs, like `print()`
---@param ... any
function M.error(...) end

return M
