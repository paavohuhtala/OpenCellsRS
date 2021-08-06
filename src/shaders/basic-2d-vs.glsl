in vec2 co;

uniform mat4 view;

void main() {
  gl_Position = view * vec4(co, 0., 1.);
}