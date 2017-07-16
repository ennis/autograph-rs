require 'data/shaders/gl'

local mesh2DColorLayout = {
	{ buffer = 0, type = GL_FLOAT, size = 2, relativeOffset = 0, normalized = false },
	{ buffer = 0, type = GL_UNSIGNED_BYTE, size = 4, relativeOffset = 8, normalized = true }
}

local mesh2DLayout = {
	{ buffer = 0, type = GL_FLOAT, size = 2, relativeOffset = 0, normalized = false },
	{ buffer = 0, type = GL_FLOAT, size = 2, relativeOffset = 8, normalized = false }
}

local mesh3DLayout = {
	{ buffer = 0, type = GL_FLOAT, size = 3, relativeOffset = 0, normalized = false },
	{ buffer = 0, type = GL_FLOAT, size = 3, relativeOffset = 12, normalized = false },
	{ buffer = 0, type = GL_FLOAT, size = 3, relativeOffset = 24, normalized = false },
	{ buffer = 0, type = GL_FLOAT, size = 2, relativeOffset = 36, normalized = false }
}

local geometryPassBase =
{
	layout = mesh3DLayout,
	rasterizerState = {
		fillMode = GL_FILL,
		cullMode = GL_BACK,
		frontFace = GL_CCW
	},
	depthStencilState = {
		depthTestEnable = true,
		depthWriteEnable = true
	},
	blendState = {
		[0] = { 
			enabled = true,
			modeRGB = GL_FUNC_ADD,
			modeAlpha = GL_FUNC_ADD,
			funcSrcRGB = GL_SRC_ALPHA,
			funcDstRGB = GL_ONE_MINUS_SRC_ALPHA,
			funcSrcAlpha = GL_ONE,
			funcDstAlpha = GL_ZERO
		},
		[1] = { enabled = false }
	},
}


local geometry2DPassBase =
{
	layout = mesh2DLayout,
	rasterizerState = {
		fillMode = GL_FILL,
		cullMode = GL_BACK,
		frontFace = GL_CCW
	},
	depthStencilState = {
		depthTestEnable = true,
		depthWriteEnable = true
	},
	blendState = {
		[0] = { 
			enabled = true,
			modeRGB = GL_FUNC_ADD,
			modeAlpha = GL_FUNC_ADD,
			funcSrcRGB = GL_SRC_ALPHA,
			funcDstRGB = GL_ONE_MINUS_SRC_ALPHA,
			funcSrcAlpha = GL_ONE,
			funcDstAlpha = GL_ZERO
		},
		[1] = { enabled = false }
	},
}

local screenPassBase = {
	layout = mesh2DLayout,
	rasterizerState = {
		fillMode = GL_FILL,
		cullMode = GL_BACK,
		frontFace = GL_CCW
	},
	blendState = {
		[0] = { 
			enabled = true,
			modeRGB = GL_FUNC_ADD,
			modeAlpha = GL_FUNC_ADD,
			funcSrcRGB = GL_SRC_ALPHA,
			funcDstRGB = GL_ONE_MINUS_SRC_ALPHA,
			funcSrcAlpha = GL_ONE,
			funcDstAlpha = GL_ZERO
		},
		[1] = { enabled = false }
	},	
	depthStencilState = {
		depthTestEnable = false,
		depthWriteEnable = false
	},
}


function deepcopy(orig)
    local orig_type = type(orig)
    local copy
    if orig_type == 'table' then
        copy = {}
        for orig_key, orig_value in next, orig, nil do
            copy[deepcopy(orig_key)] = deepcopy(orig_value)
        end
        setmetatable(copy, deepcopy(getmetatable(orig)))
    else -- number, string, boolean, etc
        copy = orig
    end
    return copy
end

function ScreenPass(table) 
	local tbl = deepcopy(screenPassBase)
	for k,v in pairs(table) do
		tbl[k] = v
	end
	return tbl
end

function GeometryPass(table) 
	local tbl = deepcopy(geometryPassBase)
	for k,v in pairs(table) do
		tbl[k] = v
	end
	return tbl
end

function Geometry2DPass(table) 
	local tbl = deepcopy(geometry2DPassBase)
	for k,v in pairs(table) do
		tbl[k] = v
	end
	return tbl
end

function ComputeShader(table)
	local tbl = {}
	tbl.isCompute = true
	for k,v in pairs(table) do
		tbl[k] = v
	end
	return tbl
end
