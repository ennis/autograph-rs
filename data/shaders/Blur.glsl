// Flexible blur shader
// Adapted from:
// http://callumhay.blogspot.com/2010/09/gaussian-blur-shader-glsl.html
#version 450
#include "Utils.glsli"
#pragma stages(compute)

layout(binding = 0, IN_FORMAT) readonly uniform image2D tex0;
layout(binding = 1, OUT_FORMAT) writeonly uniform image2D tex1;

uniform int blurSize;
uniform float sigma;

layout(local_size_x = LOCAL_SIZE_X, local_size_y = LOCAL_SIZE_Y) in;

void main() {
  ivec2 icoords = ivec2(gl_GlobalInvocationID.xy);
  float w = float(blurSize / 2);

#ifdef BLUR_H 
  ivec2 blurMultiplyVec = ivec2(1, 0);
#else 
  ivec2 blurMultiplyVec = ivec2(0, 1);
#endif

  // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 -
  // 889)
  vec3 ig;
  ig.x = 1.0 / (sqrt(TWOPI) * sigma);
  ig.y = exp(-0.5 / (sigma * sigma));
  ig.z = ig.y * ig.y;

  vec4 avgValue = vec4(0.0, 0.0, 0.0, 0.0);
  float coefficientSum = 0.0;

  // Take the central sample first...
  vec4 Sc = imageLoad(tex0, icoords);

#ifdef ALPHA_PREMULT 
  avgValue += vec4(Sc.rgb * Sc.a, Sc.a) * ig.x;
#else
  avgValue += Sc * ig.x;
#endif
  coefficientSum += ig.x;
  ig.xy *= ig.yz;

  // Go through the remaining 8 vertical samples (4 on each side of the center)
  for (int i = 1; i <= w; i++) {
    vec4 Sl = imageLoad(tex0, icoords - i * blurMultiplyVec);
    vec4 Sr = imageLoad(tex0, icoords + i * blurMultiplyVec);
    // must average alpha-premultiplied value
#ifdef ALPHA_PREMULT
    avgValue += Sl * ig.x;
    avgValue += Sr * ig.x;
#else
    avgValue += vec4(Sl.rgb * Sl.a, Sl.a) * ig.x;
    avgValue += vec4(Sr.rgb * Sr.a, Sr.a) * ig.x;
#endif
    coefficientSum += ig.x;
    ig.xy *= ig.yz;
  }

  vec4 final = avgValue / coefficientSum;

#ifndef ALPHA_PREMULT  
  // convert back to non-premultiplied alpha
  final.rgb /= final.a;
#endif

  imageStore(tex1, icoords, final);
  memoryBarrierImage();
}
