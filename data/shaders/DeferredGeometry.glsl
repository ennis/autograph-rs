#version 450
#include "Utils.glsli"

#pragma stages(vertex,fragment)

layout(std140, binding = 0) uniform CameraParameters {
  mat4 uViewMatrix;
  mat4 uProjMatrix;
  mat4 uViewProjMatrix;
  mat4 uInvProjMatrix;
  mat4 uPrevViewProjMatrixVelocity;
  mat4 uViewProjMatrixVelocity;
};

uniform int uObjectID;

#ifdef _VERTEX_

	uniform mat4 uPrevModelMatrix;
	uniform mat4 uModelMatrix;

	layout(location = 0) in vec3 iPosition;
	layout(location = 1) in vec3 iNormal;
	layout(location = 2) in vec3 iTangent;
	layout(location = 3) in vec2 iTexcoords;

	out vec3 Nv0;
	//out vec3 Pv;
	out vec3 Tv0;
	out vec2 uv;

	smooth out vec4 prevPos;
	smooth out vec4 curPos;

	void main() {
	  gl_Position = uViewProjMatrix * uModelMatrix * vec4(iPosition, 1.0f);
	  mat4 uViewModel = uViewMatrix * uModelMatrix;
	  Nv0 = (uViewModel * vec4(iNormal, 0.0)).xyz;
	  Tv0 = (uViewModel * vec4(iTangent, 0.0)).xyz;
	  //Pv = (uViewModel * vec4(iPosition, 1.0)).xyz;
	  uv = vec2(iTexcoords.x, 1-iTexcoords.y);
	  //uv = iTexcoords;
	  // positions for velocity calculation
	  prevPos = uPrevViewProjMatrixVelocity * uPrevModelMatrix * vec4(iPosition, 1.0f);
	  curPos = uViewProjMatrixVelocity * uModelMatrix * vec4(iPosition, 1.0f);
	}
#endif

#ifdef _FRAGMENT_

	in vec3 Nv0;
	//in vec3 Pv;
	in vec3 Tv0;
	in vec2 uv;

	smooth in vec4 prevPos;
	smooth in vec4 curPos;

	layout(location = 0) out vec4 rtAlbedo; 	// RGBA8
	layout(location = 1) out vec4 rtNormals;	// RG16F
	layout(location = 2) out ivec4 rtObjectID;	// RG16I
	layout(location = 3) out vec4 rtVelocity;	// RG16F

	layout(binding = 0) uniform sampler2D uAlbedo;

	void main() {
	  vec3 Nv = normalize(Nv0);
	  rtNormals = vec4(encodeNormalRG16F(Nv),0,1);
	  vec4 albedo = texture(uAlbedo, uv);
	  if (albedo.a < 0.5) 
	  	discard;
	  rtAlbedo = texture(uAlbedo, uv); 
	  rtObjectID = ivec4(uObjectID,0,0,1);

      vec2 a = curPos.xy / curPos.w;
      vec2 b = prevPos.xy / prevPos.w;
      vec2 vel = a-b;	// velocity in clip space
      rtVelocity = vec4(0.5*vel,0,1);
	}

#endif

