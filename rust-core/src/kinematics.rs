use crate::types::*;
use rayon::prelude::*;
use std::f64::consts::PI;
use std::time::Instant;

const DEG_TO_RAD: f64 = PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / PI;

#[inline(always)]
fn deg2rad(d: f64) -> f64 {
    d * DEG_TO_RAD
}

#[inline(always)]
fn rad2deg(r: f64) -> f64 {
    r * RAD_TO_DEG
}

#[inline(always)]
fn sin_d(d: f64) -> f64 {
    deg2rad(d).sin()
}

#[inline(always)]
fn cos_d(d: f64) -> f64 {
    deg2rad(d).cos()
}

pub fn inverse_kinematics(
    cp: &CartesianPoint,
    config: &MachineAxisConfig,
    tool: &ToolParameters,
    _options: IKOptions,
) -> (MachineAxisCoordinate, bool, bool) {
    let mut result = MachineAxisCoordinate {
        valid: true,
        feedrate: cp.feedrate,
        ..Default::default()
    };

    let mut singularity = false;

    let px = cp.x;
    let py = cp.y;
    let pz = cp.z;
    let a_deg = cp.a;
    let b_deg = cp.b;

    let mut a_out = a_deg;
    let mut b_out = b_deg;

    let sin_a = sin_d(a_deg);
    let cos_a = cos_d(a_deg);
    let sin_b = sin_d(b_deg);
    let cos_b = cos_d(b_deg);

    let mut tool_vec_x = sin_b;
    let mut tool_vec_y = -sin_a * cos_b;
    let mut tool_vec_z = cos_a * cos_b;

    let norm = (tool_vec_x * tool_vec_x + tool_vec_y * tool_vec_y + tool_vec_z * tool_vec_z).sqrt();
    if norm < 1e-10 {
        tool_vec_x = 0.0;
        tool_vec_y = 0.0;
        tool_vec_z = 1.0;
        singularity = true;
    } else {
        tool_vec_x /= norm;
        tool_vec_y /= norm;
        tool_vec_z /= norm;
    }

    if config.axis_type == 1 {
        let pivot_x = config.rotary_b_pivot_x;
        let pivot_y = config.rotary_b_pivot_y;
        let pivot_z = config.rotary_b_pivot_z;

        let r1 = tool.length;

        let offset_x = r1 * tool_vec_x;
        let offset_y = r1 * tool_vec_y;
        let offset_z = r1 * tool_vec_z;

        let tcp_x = px;
        let tcp_y = py;
        let tcp_z = pz;

        let mut pivot_to_tcp_x = tcp_x - pivot_x;
        let mut pivot_to_tcp_y = tcp_y - pivot_y;
        let mut pivot_to_tcp_z = tcp_z - pivot_z;

        let cos_nb = cos_b;
        let sin_nb = sin_b;
        let cos_na = cos_a;
        let sin_na = sin_a;

        let inv_b_00 = cos_nb;
        let inv_b_02 = sin_nb;
        let inv_b_10 = sin_na * sin_nb;
        let inv_b_11 = cos_na;
        let inv_b_12 = -sin_na * cos_nb;
        let inv_b_20 = -cos_na * sin_nb;
        let inv_b_21 = sin_na;
        let inv_b_22 = cos_na * cos_nb;

        let unrot_x = inv_b_00 * pivot_to_tcp_x + inv_b_02 * pivot_to_tcp_z;
        let unrot_y = inv_b_10 * pivot_to_tcp_x + inv_b_11 * pivot_to_tcp_y + inv_b_12 * pivot_to_tcp_z;
        let unrot_z = inv_b_20 * pivot_to_tcp_x + inv_b_21 * pivot_to_tcp_y + inv_b_22 * pivot_to_tcp_z;

        pivot_to_tcp_x = unrot_x;
        pivot_to_tcp_y = unrot_y;
        pivot_to_tcp_z = unrot_z;

        result.x = pivot_x + pivot_to_tcp_x - offset_x;
        result.y = pivot_y + pivot_to_tcp_y - offset_y;
        result.z = pivot_z + pivot_to_tcp_z - offset_z;
        result.a = a_out;
        result.b = b_out;
        result.c = cp.c;
    } else if config.axis_type == 2 {
        let pivot_x = config.rotary_a_pivot_x;
        let pivot_y = config.rotary_a_pivot_y;
        let pivot_z = config.rotary_a_pivot_z;

        let r1 = tool.length;

        let offset_x = r1 * tool_vec_x;
        let offset_y = r1 * tool_vec_y;
        let offset_z = r1 * tool_vec_z;

        let tcp_x = px;
        let tcp_y = py;
        let tcp_z = pz;

        let pivot_to_tcp_x = tcp_x - pivot_x - offset_x;
        let pivot_to_tcp_y = tcp_y - pivot_y - offset_y;
        let pivot_to_tcp_z = tcp_z - pivot_z - offset_z;

        result.x = pivot_x + pivot_to_tcp_x;
        result.y = pivot_y + pivot_to_tcp_y;
        result.z = pivot_z + pivot_to_tcp_z;
        result.a = a_out;
        result.b = b_out;
        result.c = cp.c;
    } else {
        result.x = px;
        result.y = py;
        result.z = pz;
        result.a = a_out;
        result.b = b_out;
        result.c = cp.c;
    }

    let mut out_of_range = false;
    if result.x < config.x_min || result.x > config.x_max {
        out_of_range = true;
    }
    if result.y < config.y_min || result.y > config.y_max {
        out_of_range = true;
    }
    if result.z < config.z_min || result.z > config.z_max {
        out_of_range = true;
    }
    if result.a < config.a_min || result.a > config.a_max {
        out_of_range = true;
    }
    if result.b < config.b_min || result.b > config.b_max {
        out_of_range = true;
    }
    if result.c < config.c_min || result.c > config.c_max {
        out_of_range = true;
    }

    if out_of_range {
        result.valid = false;
    }

    (result, singularity, out_of_range)
}

pub fn batch_inverse_kinematics(
    points: &[CartesianPoint],
    config: &MachineAxisConfig,
    tool: &ToolParameters,
    options: IKOptions,
) -> Result<(Vec<MachineAxisCoordinate>, IKMetadata), String> {
    let start = Instant::now();
    let n = points.len();

    let mut machine_coords: Vec<MachineAxisCoordinate> = vec![MachineAxisCoordinate::default(); n];
    let mut metadata = IKMetadata {
        total_points: n as u32,
        ..Default::default()
    };

    if options.parallel && n > 1000 {
        let results: Vec<(MachineAxisCoordinate, bool, bool)> = points
            .par_iter()
            .map(|p| inverse_kinematics(p, config, tool, options))
            .collect();

        for (i, (coord, sing, oor)) in results.iter().enumerate() {
            machine_coords[i] = *coord;
            if coord.valid {
                metadata.valid_points += 1;
            }
            if *sing {
                metadata.singularity_warnings += 1;
            }
            if *oor {
                metadata.out_of_range_errors += 1;
            }
        }
    } else {
        for (i, p) in points.iter().enumerate() {
            let (coord, sing, oor) = inverse_kinematics(p, config, tool, options);
            machine_coords[i] = coord;
            if coord.valid {
                metadata.valid_points += 1;
            }
            if sing {
                metadata.singularity_warnings += 1;
            }
            if oor {
                metadata.out_of_range_errors += 1;
            }
        }
    }

    metadata.solve_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    Ok((machine_coords, metadata))
}

pub fn forward_kinematics(
    mc: &MachineAxisCoordinate,
    config: &MachineAxisConfig,
    tool: &ToolParameters,
) -> CartesianPoint {
    let mut result = CartesianPoint {
        feedrate: mc.feedrate,
        ..Default::default()
    };

    let a_deg = mc.a;
    let b_deg = mc.b;
    let tool_len = tool.length;

    if config.axis_type == 1 {
        let pivot_x = config.rotary_b_pivot_x;
        let pivot_y = config.rotary_b_pivot_y;
        let pivot_z = config.rotary_b_pivot_z;

        let cos_a = cos_d(a_deg);
        let sin_a = sin_d(a_deg);
        let cos_b = cos_d(b_deg);
        let sin_b = sin_d(b_deg);

        let tool_vec_x = sin_b;
        let tool_vec_y = -sin_a * cos_b;
        let tool_vec_z = cos_a * cos_b;

        let r_00 = cos_b;
        let r_01 = sin_a * sin_b;
        let r_02 = -cos_a * sin_b;
        let r_11 = cos_a;
        let r_12 = sin_a;
        let r_20 = sin_b;
        let r_21 = -sin_a * cos_b;
        let r_22 = cos_a * cos_b;

        let mx = mc.x;
        let my = mc.y;
        let mz = mc.z;

        let tip_to_pivot_x = mx - pivot_x;
        let tip_to_pivot_y = my - pivot_y;
        let tip_to_pivot_z = mz - pivot_z;

        let rotated_x = r_00 * tip_to_pivot_x + r_01 * tip_to_pivot_y + r_02 * tip_to_pivot_z;
        let rotated_y = r_11 * tip_to_pivot_y + r_12 * tip_to_pivot_z;
        let rotated_z = r_20 * tip_to_pivot_x + r_21 * tip_to_pivot_y + r_22 * tip_to_pivot_z;

        result.x = pivot_x + rotated_x + tool_len * tool_vec_x;
        result.y = pivot_y + rotated_y + tool_len * tool_vec_y;
        result.z = pivot_z + rotated_z + tool_len * tool_vec_z;
    } else if config.axis_type == 2 {
        let pivot_x = config.rotary_a_pivot_x;
        let pivot_y = config.rotary_a_pivot_y;
        let pivot_z = config.rotary_a_pivot_z;

        let cos_a = cos_d(a_deg);
        let sin_a = sin_d(a_deg);
        let cos_b = cos_d(b_deg);
        let sin_b = sin_d(b_deg);

        let tool_vec_x = sin_b;
        let tool_vec_y = -sin_a * cos_b;
        let tool_vec_z = cos_a * cos_b;

        let mx = mc.x;
        let my = mc.y;
        let mz = mc.z;

        result.x = pivot_x + (mx - pivot_x) + tool_len * tool_vec_x;
        result.y = pivot_y + (my - pivot_y) + tool_len * tool_vec_y;
        result.z = pivot_z + (mz - pivot_z) + tool_len * tool_vec_z;
    } else {
        result.x = mc.x;
        result.y = mc.y;
        result.z = mc.z;
    }

    result.a = a_deg;
    result.b = b_deg;
    result.c = mc.c;

    result
}

pub fn batch_forward_kinematics(
    coords: &[MachineAxisCoordinate],
    config: &MachineAxisConfig,
    tool: &ToolParameters,
) -> Result<Vec<CartesianPoint>, String> {
    let result: Vec<CartesianPoint> = coords
        .par_iter()
        .map(|mc| forward_kinematics(mc, config, tool))
        .collect();
    Ok(result)
}

pub fn check_singularity(a_deg: f64, b_deg: f64, threshold: f64) -> bool {
    let cos_b = cos_d(b_deg);
    cos_b.abs() < threshold
}

pub fn normalize_angle(angle: f64, min: f64, max: f64) -> f64 {
    let range = max - min;
    if range <= 0.0 {
        return angle;
    }
    let mut result = angle;
    while result > max {
        result -= 360.0;
    }
    while result < min {
        result += 360.0;
    }
    result
}
