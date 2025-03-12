#version 410 core

uniform vec4 baseColor;
uniform sampler2D colorMap;
uniform bool hasColorMap;

out vec4 FragColor;

in vec2 TexCoord;

void main()
{
    if(hasColorMap) {
        FragColor = baseColor * texture(colorMap, TexCoord);
    } else {    
        FragColor = baseColor;
    }
} 
