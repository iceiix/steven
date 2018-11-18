uniform sampler2DMS tcolor;

out vec4 fragColor;

void main() {
    ivec2 C = ivec2(gl_FragCoord.xy);
    vec4 col = texelFetch(tcolor, C, 0);

    fragColor = vec4(col.rgb, 0.0);
}
