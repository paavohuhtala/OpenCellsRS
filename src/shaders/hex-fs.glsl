in float v_edginess;

out vec4 frag;

uniform vec3 model_color;

const vec4 border_color = vec4(1.0, 1.0, 1.0, 1.0);

void main() {
  vec4 hex_color = vec4(model_color, 1.0);

  if (v_edginess > 0.95) {
    frag = mix(hex_color, border_color, 0.95);
  }
  else if (v_edginess < 0.8) {
    frag = hex_color;
  }
  else {
    float clamped_edginess = clamp(pow(v_edginess, 10.0), 0.0, 1.0);
    frag = mix(hex_color, border_color, clamped_edginess);
  }
}