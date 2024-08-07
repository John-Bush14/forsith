#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 vPosition;
layout(location = 1) in vec4 vColor;

layout(binding = 0) uniform CameraBufferObject {mat4 view;} cam;
layout(binding = 1) uniform DrawableBufferObject {mat4 model;} draw;

layout(location = 0) out vec4 fragColor;

void main() {
    gl_Position = cam.view * (vec4(vPosition, 0.0, 1.0) * draw.model);
    fragColor = vColor;
}
