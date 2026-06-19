use napi_derive::napi;
use std::default::Default;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CartesianPointRepr {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub feedrate: f64,
    pub spindle: f64,
}

impl CartesianPointRepr {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MachineCoordRepr {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub feedrate: f64,
    pub padding: f64,
}

impl MachineCoordRepr {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct CartesianPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub feedrate: f64,
    pub spindle: f64,
}

impl Default for CartesianPoint {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            feedrate: 0.0,
            spindle: 0.0,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct MachineAxisCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub feedrate: f64,
    pub valid: bool,
}

impl Default for MachineAxisCoordinate {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            feedrate: 0.0,
            valid: true,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct GCodeBlock {
    pub line_number: u32,
    pub block_number: u32,
    pub g_code: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub feedrate: f64,
    pub spindle: f64,
    pub is_rapid: bool,
    pub is_motion: bool,
    pub tool_number: u32,
}

impl Default for GCodeBlock {
    fn default() -> Self {
        Self {
            line_number: 0,
            block_number: 0,
            g_code: 0,
            x: f64::NAN,
            y: f64::NAN,
            z: f64::NAN,
            a: f64::NAN,
            b: f64::NAN,
            c: f64::NAN,
            feedrate: f64::NAN,
            spindle: f64::NAN,
            is_rapid: false,
            is_motion: false,
            tool_number: 0,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ParseMetadata {
    pub total_lines: u32,
    pub total_blocks: u32,
    pub motion_blocks: u32,
    pub rapid_blocks: u32,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub total_distance: f64,
    pub estimated_time: f64,
    pub parse_time_ms: f64,
}

impl Default for ParseMetadata {
    fn default() -> Self {
        Self {
            total_lines: 0,
            total_blocks: 0,
            motion_blocks: 0,
            rapid_blocks: 0,
            min_x: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
            min_z: f64::INFINITY,
            max_z: f64::NEG_INFINITY,
            total_distance: 0.0,
            estimated_time: 0.0,
            parse_time_ms: 0.0,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ParsedGCodeResult {
    pub blocks: Vec<GCodeBlock>,
    pub metadata: ParseMetadata,
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct IKOptions {
    pub parallel: bool,
    pub enable_rtcp: bool,
    pub singularity_threshold: f64,
    pub max_iterations: u32,
}

impl Default for IKOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            enable_rtcp: true,
            singularity_threshold: 0.001,
            max_iterations: 100,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct IKMetadata {
    pub total_points: u32,
    pub valid_points: u32,
    pub singularity_warnings: u32,
    pub out_of_range_errors: u32,
    pub solve_time_ms: f64,
}

impl Default for IKMetadata {
    fn default() -> Self {
        Self {
            total_points: 0,
            valid_points: 0,
            singularity_warnings: 0,
            out_of_range_errors: 0,
            solve_time_ms: 0.0,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct IKResult {
    pub machine_coords: Vec<MachineAxisCoordinate>,
    pub metadata: IKMetadata,
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct MachineAxisConfig {
    pub axis_type: u32,
    pub rotary_a_pivot_x: f64,
    pub rotary_a_pivot_y: f64,
    pub rotary_a_pivot_z: f64,
    pub rotary_b_pivot_x: f64,
    pub rotary_b_pivot_y: f64,
    pub rotary_b_pivot_z: f64,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
    pub a_min: f64,
    pub a_max: f64,
    pub b_min: f64,
    pub b_max: f64,
    pub c_min: f64,
    pub c_max: f64,
    pub a_is_table: bool,
    pub b_is_table: bool,
    pub c_is_table: bool,
}

impl Default for MachineAxisConfig {
    fn default() -> Self {
        Self {
            axis_type: 1,
            rotary_a_pivot_x: 0.0,
            rotary_a_pivot_y: 0.0,
            rotary_a_pivot_z: 0.0,
            rotary_b_pivot_x: 0.0,
            rotary_b_pivot_y: 0.0,
            rotary_b_pivot_z: 300.0,
            x_min: -500.0,
            x_max: 500.0,
            y_min: -400.0,
            y_max: 400.0,
            z_min: 0.0,
            z_max: 500.0,
            a_min: -120.0,
            a_max: 120.0,
            b_min: -360.0,
            b_max: 360.0,
            c_min: -360.0,
            c_max: 360.0,
            a_is_table: true,
            b_is_table: false,
            c_is_table: false,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct ToolParameters {
    pub length: f64,
    pub diameter: f64,
    pub radius: f64,
    pub tip_angle: f64,
    pub tool_number: u32,
}

impl Default for ToolParameters {
    fn default() -> Self {
        Self {
            length: 100.0,
            diameter: 10.0,
            radius: 5.0,
            tip_angle: 0.0,
            tool_number: 1,
        }
    }
}
