out vec4 frag;

uniform vec3 model_color;

void main() {
  frag = vec4(model_color, 1.0);
}