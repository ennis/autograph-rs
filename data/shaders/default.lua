require 'data/shaders/utils'

geometryPass = GeometryPass
{
	shaderFile = 'DeferredGeometry.glsl'
}

deferredPass = ScreenPass
{
	shaderFile = 'DeferredDebug.glsl',
	blendState = {
		[0] = { enabled = false },
		[1] = { enabled = false },
		[2] = { enabled = false },
		[3] = { enabled = false },
		[4] = { enabled = false }}
}

TAA_Average = ComputeShader 
{
	shaderFile = 'TAA_Average.glsl',
	barrierBits = bit.bor(GL_TEXTURE_FETCH_BARRIER_BIT, GL_SHADER_IMAGE_ACCESS_BARRIER_BIT)
}

drawSprite = Geometry2DPass
{
	depthStencilState = {
		depthTestEnable = false,
		depthWriteEnable = false
	},
	shaderFile = 'DrawSprite.glsl'
}

textureView = Geometry2DPass
{
	depthStencilState = {
		depthTestEnable = false,
		depthWriteEnable = false
	},
	shaderFile = 'TextureView.glsl'
}


drawMeshDefault = GeometryPass
{
	depthStencilState = {
		depthTestEnable = true,
		depthWriteEnable = true
	},
	shaderFile = 'DrawMesh.glsl'
}

drawWireMesh = GeometryPass
{
	rasterizerState = {
		fillMode = GL_LINE,
	},
	depthStencilState = {
		depthTestEnable = true,
		depthWriteEnable = false
	},
	shaderFile = 'DrawWire.glsl'
}

drawWireMeshNoDepth = GeometryPass
{
	rasterizerState = {
		fillMode = GL_LINE,
	},
	depthStencilState = {
		depthTestEnable = false,
		depthWriteEnable = false
	},
	shaderFile = 'DrawWire.glsl'
}


drawWireMesh2DColor = GeometryPass
{
	layout = {
		{ buffer = 0, type = GL_FLOAT, size = 2, relativeOffset = 0, normalized = false },
		{ buffer = 0, type = GL_UNSIGNED_BYTE, size = 4, relativeOffset = 8, normalized = true }
	},
	rasterizerState = {
		fillMode = GL_LINE,
	},
	depthStencilState = {
		depthTestEnable = false,
		depthWriteEnable = false
	},
	shaderFile = 'DrawMesh2DColor.glsl'
}

----------------------------------------------------------
-- Blur passes
local function BlurPasses(defs)
	local H = ComputeShader {
		defines = deepcopy(defs),
		shaderFile = 'Blur.glsl',
		barrierBits = bit.bor(GL_TEXTURE_FETCH_BARRIER_BIT, GL_SHADER_IMAGE_ACCESS_BARRIER_BIT)
	}
	local V = ComputeShader {
		defines = deepcopy(defs),
		shaderFile = 'Blur.glsl',
		barrierBits = bit.bor(GL_TEXTURE_FETCH_BARRIER_BIT, GL_SHADER_IMAGE_ACCESS_BARRIER_BIT)
	}
	H.defines.BLUR_H = 1
	V.defines.BLUR_V = 1
	return H,V
end

blurH_RGBA16F, blurV_RGBA16F = BlurPasses
{
	IN_FORMAT = rgba16f,
	OUT_FORMAT = rgba16f,
	ALPHA_PREMULT = 0
}

blurH_RGBA16F_AlphaPremult, blurV_RGBA16F_AlphaPremult = BlurPasses
{
	IN_FORMAT = rgba16f,
	OUT_FORMAT = rgba16f,
	ALPHA_PREMULT = 1
}

blurH_RGBA8, blurV_RGBA8 = BlurPasses
{
	IN_FORMAT = rgba16f,
	OUT_FORMAT = rgba16f,
	ALPHA_PREMULT = 0
}

blurH_RGBA8_AlphaPremult, blurV_RGBA8_AlphaPremult = BlurPasses
{
	IN_FORMAT = rgba8,
	OUT_FORMAT = rgba8,
	ALPHA_PREMULT = 1
}

blurH_R32F, blurV_R32F = BlurPasses
{
	IN_FORMAT = r32f,
	OUT_FORMAT = r32f,
	ALPHA_PREMULT = 1
}
