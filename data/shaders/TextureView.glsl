#version 450
#include "Utils.glsli"
#include "Blend.glsli"
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

uniform float uLod;
uniform vec2 uRange;
uniform vec4 uBorder;

void main() {
  // checkerboard texture
  vec2 uv = fTexcoord;
  if (uv.x > 1.0f || uv.x < 0.0f || uv.y > 1.0f || uv.y < 0.0f) {
  	color = uBorder;
  	return;
  }
  ivec2 res = textureSize(tex,0);
  uv.x *= float(res.x) / float(res.y);
  vec4 c = texture(tex, fTexcoord.xy);
  color = blend(
  	vec4(remap(uRange.x, uRange.y, c.rgb), c.a),
  	mix(vec4(0.5,0.5,0.5,1.0), vec4(0.8,0.8,0.8,1.0), checker(uv, res.x/20.0f)));
}

#endif