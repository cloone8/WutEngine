#version 330 core

in vec3 wuteng_Position;

uniform mat4 wuteng_objectToWorld;

void main()
{
    gl_Position = vec4(wuteng_Position, 1.0);
}
