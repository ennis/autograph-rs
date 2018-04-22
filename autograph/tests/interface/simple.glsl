#version 450
#pragma stages(vertex,fragment)
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// visible to all stages
layout(location=0) uniform float A;

#ifdef _VERTEX_

// visible to this stage only
layout(location=1) uniform mat3 B;

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
