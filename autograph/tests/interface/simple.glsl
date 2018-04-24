#version 450
#pragma stages(vertex,fragment)
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// visible to all stages
layout(location=0) uniform float A;
layout(binding=0) uniform sampler2D tex;

layout(binding=0,std140) uniform U {
	mat4 viewMatrix;
	mat4 projMatrix;
	mat4 viewProjMatrix;
	mat4 invViewProjMatrix;
	mat4 prevViewProjMatrixVelocity;
	mat4 viewProjMatrixVelocity;
	ivec2 temporalAAOffset;
};

layout(binding=0,std430) buffer B {
	int data[];
};

#ifdef _VERTEX_

// visible to this stage only
layout(location=1) uniform float b;

void main() {
  gl_Position = vec4(0.0);
}
#endif

#ifdef _FRAGMENT_
layout(location = 0) out vec4 color;

void main() {
    color = vec4(0.0);
}
#endif
