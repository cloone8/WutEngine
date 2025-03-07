#version 410 core

layout(location = 0) in vec3 wuteng_Position;

uniform mat4 wuteng_ModelMat;
uniform mat4 wuteng_ViewMat;
uniform mat4 wuteng_ProjectionMat;

void main()
{
    gl_Position = wuteng_ProjectionMat * wuteng_ViewMat * wuteng_ModelMat * vec4(wuteng_Position, 1.0);
}
