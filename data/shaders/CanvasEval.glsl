#version 450
#include "Utils.glsli"
#include "ImageShader.glsli"

layout(binding = 0) uniform sampler2D t_normals;
layout(binding = 1) uniform usampler2D t_coefs0;
layout(binding = 2) uniform usampler2D t_coefs1;

uniform vec2 u_ref;

vec4 image(vec2 texcoords)
{
  ivec2 T = ivec2(texcoords*textureSize(t_normals,0));

  vec3 N = unpackNormal(texelFetch(t_normals, T, 0));
  uvec4 C0 = texelFetch(t_coefs0, T, 0);
  uvec4 C1 = texelFetch(t_coefs1, T, 0);

  vec2 a0a1 = unpackUnorm2x16(C0.x);
  vec2 a2b0 = unpackUnorm2x16(C0.y);
  vec2 b1b2 = unpackUnorm2x16(C0.y);
  vec2 c0c1 = unpackUnorm2x16(C0.w);
  vec2 c2d0 = unpackUnorm2x16(C1.x);
  vec2 d1d2 = unpackUnorm2x16(C1.y);
  vec2 refp = unpackUnorm2x16(C1.z);

  vec3 a = vec3(a0a1, a2b0.x);
  vec3 b = vec3(a2b0.y, b1b2.x, b1b2.y);
  vec3 c = vec3(c0c1, c2d0.x);
  vec3 d = vec3(c2d0.y, d1d2.x, d1d2.y);

  return vec4(cosPalette(u_ref.x + texcoords.x, a, b, c, d),1.0);
}

