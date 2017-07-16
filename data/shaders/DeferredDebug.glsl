#version 450
#include "Utils.glsli"
#include "ImageShader.glsli"

///////////////////////////////////////////////////////
///////////////////////////////////////////////////////
///////////////////////////////////////////////////////
layout(binding = 0) uniform sampler2D uAlbedo;
layout(binding = 1) uniform sampler2D uNormals;
layout(binding = 2) uniform isampler2D uObjectID;
layout(binding = 3) uniform sampler2D uVelocity;
layout(binding = 4) uniform sampler2D uDepth;

layout(std140, binding = 0) uniform CameraParameters {
  mat4 uViewMatrix;
  mat4 uProjMatrix;
  mat4 uViewProjMatrix;
  mat4 uInvProjMatrix;
  mat4 uPrevViewProjMatrixVelocity;
  mat4 uViewProjMatrixVelocity;
};

uniform int uDebugMode; 

///////////////////////////////////////////////////////
///////////////////////////////////////////////////////
///////////////////////////////////////////////////////
vec3 VSPositionFromDepth(vec2 texcoord)
{
    // Get the depth value for this pixel
    float z = texture(uDepth, texcoord).x;  
    // Get x/w and y/w from the viewport position
    float x = texcoord.x * 2 - 1;
    float y = (1 - texcoord.y) * 2 - 1;
    vec4 vProjectedPos = vec4(x, y, z, 1.0f);
    // Transform by the inverse projection matrix
    vec4 vPositionVS = uInvProjMatrix * vProjectedPos;  
    // Divide by w to get the view-space position
    return vPositionVS.xyz / vPositionVS.w;  
}

vec4 image(vec2 uv)
{
  ivec2 screenPos = ivec2(uv*textureSize(uAlbedo,0));
  vec4 c = vec4(0,0,0,1);
  if (uDebugMode == 1) {
    // normals
    vec4 encNv = texelFetch(uNormals, screenPos, 0);
    vec3 Nv = decodeNormalRG16F(encNv.xy);
    c = packNormal(Nv);
  }
  else if (uDebugMode == 2) {
    // object ID
    uint id = uint(texelFetch(uObjectID, screenPos, 0).r);
    c = vec4(0.333*vec3(
      float(bitfieldExtract(id,0,1) + bitfieldExtract(id,3,1) + bitfieldExtract(id,6,1)),
      float(bitfieldExtract(id,1,1) + bitfieldExtract(id,4,1) + bitfieldExtract(id,7,1)),
      float(bitfieldExtract(id,2,1) + bitfieldExtract(id,5,1) + bitfieldExtract(id,8,1))),
      1.0);
  }
  else if (uDebugMode == 3) {
    // view-space depth
    vec3 vPos = VSPositionFromDepth(uv);
    c = vec4(vPos.zzz, 1);
  } 
  else if (uDebugMode == 4) {
    // view-space positions
    vec3 vPos = VSPositionFromDepth(uv);
    c = vec4(vPos, 1);
  }
  else if (uDebugMode == 5) {
    // albedo
    c = texelFetch(uAlbedo, screenPos, 0);
  }
  else if (uDebugMode == 6) {
    // velocity
    c = texelFetch(uVelocity, screenPos, 0);
  }
  return c;
}

