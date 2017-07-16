#version 450
#include "Utils.glsli"
#include "MeshShader.glsli"

#pragma stages(vertex,fragment)


#ifdef _VERTEX_

out vec3 Nv0;
out vec3 Pv;

void main() {
  gl_Position = uViewProjMatrix * uModelMatrix * vec4(iPosition, 1.0);
  // assume no scaling in modelMatrix
  Nv0 = (uViewMatrix * uModelMatrix * vec4(iNormal, 0.0)).xyz;
  Pv = (uViewMatrix * uModelMatrix * vec4(iPosition, 1.0)).xyz;
}
#endif

#ifdef _FRAGMENT_

in vec3 Nv0;
in vec3 Pv;

uniform vec4 uColor;

layout(location = 0) out vec4 color;

void main() {
  vec3 Nv = normalize(Nv0);
  vec3 V = -normalize(Pv);
  color = vec4(uColor.rgb*dot(Nv,V), uColor.a);
}

#endif

