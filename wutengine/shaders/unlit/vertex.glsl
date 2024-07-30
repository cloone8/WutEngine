#version 330 core

in vec3 wuteng_Position;

void main()
{
    gl_Position = vec4(wuteng_Position, 1.0);
}
