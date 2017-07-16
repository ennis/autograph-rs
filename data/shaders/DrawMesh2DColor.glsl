#version 450
#pragma stages(vertex,fragment)
#ifdef _VERTEX_

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

out vec4 fColor;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  fColor = color;
}
#endif

#ifdef _FRAGMENT_

in vec4 fColor;
layout(location = 0) out vec4 color;

void main() {
  color = fColor;
}

#endif

