use crate::types::*;
use napi::*;
use napi_derive::napi;

pub unsafe fn write_cartesian_points_to_buffer(
    data_ptr: *mut CartesianPointRepr,
    points: &[CartesianPoint],
    capacity: usize,
) {
    let len = points.len().min(capacity);
    for i in 0..len {
        let src = &points[i];
        let dst = &mut *data_ptr.add(i);
        dst.x = src.x;
        dst.y = src.y;
        dst.z = src.z;
        dst.a = src.a;
        dst.b = src.b;
        dst.c = src.c;
        dst.feedrate = src.feedrate;
        dst.spindle = src.spindle;
    }
}

pub unsafe fn write_machine_coords_to_buffer(
    data_ptr: *mut MachineCoordRepr,
    coords: &[MachineAxisCoordinate],
    capacity: usize,
) {
    let len = coords.len().min(capacity);
    for i in 0..len {
        let src = &coords[i];
        let dst = &mut *data_ptr.add(i);
        dst.x = src.x;
        dst.y = src.y;
        dst.z = src.z;
        dst.a = src.a;
        dst.b = src.b;
        dst.c = src.c;
        dst.feedrate = src.feedrate;
        dst.padding = if src.valid { 1.0 } else { 0.0 };
    }
}

#[napi(object)]
pub struct PipelineResultShared {
    pub cartesian_buffer: JsArrayBuffer,
    pub machine_buffer: JsArrayBuffer,
    pub point_count: u32,
    pub parse_metadata: ParseMetadata,
    pub ik_metadata: IKMetadata,
}

#[napi(object)]
pub struct CartesianFieldOffsets {
    pub x_offset: u32,
    pub y_offset: u32,
    pub z_offset: u32,
    pub a_offset: u32,
    pub b_offset: u32,
    pub c_offset: u32,
    pub feedrate_offset: u32,
    pub spindle_offset: u32,
    pub stride: u32,
}

#[napi(object)]
pub struct MachineFieldOffsets {
    pub x_offset: u32,
    pub y_offset: u32,
    pub z_offset: u32,
    pub a_offset: u32,
    pub b_offset: u32,
    pub c_offset: u32,
    pub feedrate_offset: u32,
    pub valid_offset: u32,
    pub stride: u32,
}

#[napi]
pub fn get_cartesian_field_offsets() -> CartesianFieldOffsets {
    CartesianFieldOffsets {
        x_offset: 0,
        y_offset: 8,
        z_offset: 16,
        a_offset: 24,
        b_offset: 32,
        c_offset: 40,
        feedrate_offset: 48,
        spindle_offset: 56,
        stride: CartesianPointRepr::SIZE as u32,
    }
}

#[napi]
pub fn get_machine_field_offsets() -> MachineFieldOffsets {
    MachineFieldOffsets {
        x_offset: 0,
        y_offset: 8,
        z_offset: 16,
        a_offset: 24,
        b_offset: 32,
        c_offset: 40,
        feedrate_offset: 48,
        valid_offset: 56,
        stride: MachineCoordRepr::SIZE as u32,
    }
}

pub fn create_pipeline_result(
    env: Env,
    cartesian_points: &[CartesianPoint],
    machine_coords: &[MachineAxisCoordinate],
    parse_meta: ParseMetadata,
    ik_meta: IKMetadata,
) -> Result<PipelineResultShared> {
    let count = cartesian_points.len();

    let cart_byte_len = count * CartesianPointRepr::SIZE;
    let mut cart_data: Vec<u8> = vec![0u8; cart_byte_len];
    let cart_ptr = cart_data.as_mut_ptr() as *mut CartesianPointRepr;
    unsafe {
        write_cartesian_points_to_buffer(cart_ptr, cartesian_points, count);
    }
    let cart_buf_value = env.create_arraybuffer_with_data(cart_data)?;
    let cart_buffer = cart_buf_value.into_raw();

    let mach_byte_len = count * MachineCoordRepr::SIZE;
    let mut mach_data: Vec<u8> = vec![0u8; mach_byte_len];
    let mach_ptr = mach_data.as_mut_ptr() as *mut MachineCoordRepr;
    unsafe {
        write_machine_coords_to_buffer(mach_ptr, machine_coords, count);
    }
    let mach_buf_value = env.create_arraybuffer_with_data(mach_data)?;
    let mach_buffer = mach_buf_value.into_raw();

    Ok(PipelineResultShared {
        cartesian_buffer: cart_buffer,
        machine_buffer: mach_buffer,
        point_count: count as u32,
        parse_metadata: parse_meta,
        ik_metadata: ik_meta,
    })
}
