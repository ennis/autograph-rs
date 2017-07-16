#version 450
#include "MeshShader.glsli"
#pragma stages(vertex,fragment)

#ifdef _VERTEX_

out vec3 Nw0;
out vec3 Pv;
out vec3 Tv0;
out vec2 fTexcoords;

void main() {
  gl_Position = uViewProjMatrix * uModelMatrix * vec4(iPosition, 1.0f);
  // assume no scaling in modelMatrix
  Nw0 = (uModelMatrix * vec4(iNormal, 0.0)).xyz;
  Tv0 = (uViewMatrix * uModelMatrix * vec4(iTangent, 0.0)).xyz;
  Pv = (uViewMatrix * uModelMatrix * vec4(iPosition, 1.0)).xyz;
  fTexcoords = iTexcoords;
}
#endif

#ifdef _FRAGMENT_ 

in vec3 Nw0;
in vec3 Pv;
in vec3 Tv0;
in vec2 fTexcoords;

layout(location = 0) out vec4 rtNormals;
layout(location = 1) out vec4 rtDiffuse;

void main() {
	vec3 Nw = normalize(Nw0);
  rtNormals = packNormals(Nw);
  rtDiffuse = vec4(1.0);  // TODO
}

#endif

