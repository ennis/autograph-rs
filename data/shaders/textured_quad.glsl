#version 450
//#extension GL_ARB_separate_shader_objects : enable
//#extension GL_ARB_shading_language_420pack : enable
#pragma stages(vertex,fragment)
#pragma input_layout(rg32f,0,0,rg32f,0,8)
#pragma primitive_topology(triangle)

#ifdef _VERTEX_

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 texcoord;
layout(location = 0) out vec2 f_uv;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  f_uv = texcoord;
}
#endif

#ifdef _FRAGMENT_
layout(binding = 0) uniform sampler2D tex;
layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

void main() {
    color = texture(tex,f_uv);
}
#endif