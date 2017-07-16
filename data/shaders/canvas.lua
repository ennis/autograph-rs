require 'shaders/utils'

gbuffer = GeometryPass{
	shaderFile = 'CanvasGBuffers.glsl'
}

evaluation = ScreenPass{
	shaderFile = 'CanvasEval.glsl'
}

