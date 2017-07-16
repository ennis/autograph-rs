#version 450
layout(binding = 0) uniform sampler2D tileset;
layout(binding = 1) uniform sampler2D tilemap;

#pragma stages(vertex,fragment)

#ifdef _VERTEX_

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 texcoord;

layout(binding=0) uniform U {
	vec2 offset;
	ivec2 tileOffset;	// in tiles
	ivec2 tileSize;			// in pixels 
	ivec2 viewportSize;		// in pixels
	ivec2 gridSizeTiles;	// in tiles
};

centroid out vec2 fTexcoord;

void main() {
  vec2 pos2 = 2.0f * ((pos*tileSize - offset) / viewportSize - 0.5f);
  pos2.y = -pos2.y;
  gl_Position = vec4(pos2, 0.0, 1.0);
  // get tile 
  int tile = gl_VertexID / 4;
  int corner = gl_VertexID % 4;
  ivec2 tilePos = tileOffset + ivec2(tile % gridSizeTiles.x, tile / gridSizeTiles.x);
  vec2 tileTexcoords = texelFetch(tilemap, tilePos, 0).rg;
  fTexcoord = tileTexcoords + vec2(tileSize)/textureSize(tileset,0) * vec2(corner & 1, (corner >> 1) & 1);
}
#endif

#ifdef _FRAGMENT_

layout(location = 0) out vec4 color;
centroid in vec2 fTexcoord;

void main() {
  color = texture(tileset, fTexcoord);
  //color = vec4(fTexcoord, 1.0, 1.0);
}

#endif

