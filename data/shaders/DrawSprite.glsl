#version 450
#include "Utils.glsli"

#pragma stages(vertex,fragment)

#ifdef _VERTEX_ 

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 texcoord;
out vec2 fTexcoord;
void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  fTexcoord = texcoord;
}
#endif

#ifdef _FRAGMENT_

layout(binding = 0) uniform sampler2D tex;
layout(location = 0) out vec4 color;
in vec2 fTexcoord;

void main() {
  color = texture(tex, fTexcoord.xy);
}

#endif

