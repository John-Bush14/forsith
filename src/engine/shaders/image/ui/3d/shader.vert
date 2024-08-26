#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 vPosition;
layout(location = 1) in vec4 vColor;
layout(location = 2) in vec2 vCoords;

layout(binding = 0) uniform CameraBufferObject {mat4 view; float proj;} cam;
layout(binding = 1) uniform DrawableBufferObject {mat4 model;} draw;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragCoords;

void main() {
   gl_Position = cam.proj * cam.view * (vec4(vPosition, 1.0) * draw.model);

   fragColor = vColor;
   fragCoords = vCoords;
}
