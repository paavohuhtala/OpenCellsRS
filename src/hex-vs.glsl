in vec2 co;
in float edginess;

out float v_edginess;
uniform mat4 view;

void main() {
  gl_Position = view * vec4(co, 0., 1.);
  v_edginess = edginess;
}