struct InstanceConstants {
    model: mat4x4<f32>,
    mvp: mat4x4<f32>,
}

@group(WUTENGINE_INSTANCE_GROUP) @binding(0) var<uniform> instance_params: InstanceConstants;
