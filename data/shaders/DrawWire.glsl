#version 450
#include "Utils.glsli"
#include "MeshShader.glsli"

#pragma stages(vertex,fragment)

#ifdef _VERTEX_
void main() {
  gl_Position = uViewProjMatrix * uModelMatrix * vec4(iPosition, 1.0f);
}
#endif

#ifdef _FRAGMENT_

uniform vec4 uWireColor;

layout(location = 0) out vec4 color;

void main() {
  color = uWireColor;
}
#endif
