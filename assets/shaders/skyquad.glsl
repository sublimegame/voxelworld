#version 330 core

layout(location = 0) in vec4 pos;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;

out vec3 fragpos;
out vec2 tc;

void main() {
	vec4 p = vec4(pos.x, 0.0, pos.y, 1.0);
	vec4 glpos = persp * view * transform * p;
	tc = pos.xy * 0.5 + vec2(0.5, 0.5);
	tc = tc.yx;
	tc.y = 1.0 - tc.y;
	gl_Position = glpos.xyww;
	fragpos = (transform * p).xyz;
}
