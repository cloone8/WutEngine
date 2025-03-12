#version 410 core

layout(location = 0) in vec3 wuteng_Position;
layout(location = 1) in vec2 wuteng_UV;

uniform mat4 wuteng_ModelMat;
uniform mat4 wuteng_ViewMat;
uniform mat4 wuteng_ProjectionMat;

out vec2 TexCoord;

void main()
{
    gl_Position = wuteng_ProjectionMat * wuteng_ViewMat * wuteng_ModelMat * vec4(wuteng_Position, 1.0);
    TexCoord = wuteng_UV;
}
