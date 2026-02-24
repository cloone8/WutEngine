struct InstanceConstants {
    model: mat4x4<f32>,
    mvp: mat4x4<f32>,
}

@group(1) @binding(0) var instance_params: InstanceConstants;
