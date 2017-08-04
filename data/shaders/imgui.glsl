#version 450

#pragma stages(vertex,fragment)
#pragma input_layout(rg32f,0,0,rg32f,0,8,rgba32f,0,16)
#pragma primitive_topology(triangle)

#ifdef _VERTEX_
layout(binding=0) uniform mat4 matrix;
layout(location=0) in vec2 pos;
layout(location=1) in vec2 uv;
layout(location=2) in vec4 col;

out vec2 f_uv;
out vec4 f_color;

void main() {
  f_uv = uv;
  f_color = col / 255.0;
  gl_Position = matrix * vec4(pos.xy, 0, 1);
}
#endif

#ifdef _FRAGMENT_
layout(binding=0) uniform sampler2D tex;

in vec2 f_uv;
in vec4 f_color;

out vec4 out_color;

void main() {
  out_color = f_color * texture(tex, f_uv.st);
}
#endif
