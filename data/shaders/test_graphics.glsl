#version 450
#pragma stages(vertex,fragment)
#pragma input_layout(rg32f,0,0)
#pragma primitive_topology(triangle)

#ifdef _VERTEX_

layout(location = 0) in vec2 pos;
out vec2 uv;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  uv = 0.5*(pos+1.0);
}

#endif

#ifdef _FRAGMENT_

in vec2 uv;
out vec4 color;

void main()
{
    color = vec4(uv,0,1);
}

#endif