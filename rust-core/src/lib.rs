#![deny(clippy::all)]
#![allow(clippy::too_many_arguments)]

use napi::*;
use napi_derive::napi;

pub mod types;
pub mod gcode_parser;
pub mod kinematics;
pub mod shared_memory;
pub mod voxel_engine;

use types::*;
use shared_memory::*;
use voxel_engine::*;

#[napi]
pub struct CncSimulator {
    axis_config: MachineAxisConfig,
    tool_params: ToolParameters,
}

#[napi]
impl CncSimulator {
    #[napi(constructor)]
    pub fn new(config: Option<MachineAxisConfig>) -> Self {
        Self {
            axis_config: config.unwrap_or_default(),
            tool_params: ToolParameters::default(),
        }
    }

    #[napi]
    pub fn set_tool_parameters(&mut self, params: ToolParameters) {
        self.tool_params = params;
    }

    #[napi]
    pub fn set_machine_config(&mut self, config: MachineAxisConfig) {
        self.axis_config = config;
    }

    #[napi]
    pub fn parse_gcode_sync(&self, gcode_text: String) -> Result<ParsedGCodeResult> {
        let (blocks, metadata) = gcode_parser::parse_gcode(&gcode_text)
            .map_err(|e| Error::from_reason(format!("G-code parse failed: {}", e)))?;
        Ok(ParsedGCodeResult {
            blocks,
            metadata,
        })
    }

    #[napi]
    pub fn parse_gcode_mmap(&self, file_path: String) -> Result<ParsedGCodeResult> {
        let (blocks, metadata) = gcode_parser::parse_gcode_mmap(&file_path)
            .map_err(|e| Error::from_reason(format!("G-code file parse failed: {}", e)))?;
        Ok(ParsedGCodeResult {
            blocks,
            metadata,
        })
    }

    #[napi]
    pub fn inverse_kinematics_sync(
        &self,
        cartesian_points: Vec<CartesianPoint>,
        params: Option<IKOptions>,
    ) -> Result<IKResult> {
        let options = params.unwrap_or_default();
        let (machine_coords, metadata) = kinematics::batch_inverse_kinematics(
            &cartesian_points, &self.axis_config, &self.tool_params, options
        ).map_err(|e| Error::from_reason(format!("IK solver failed: {}", e)))?;
        Ok(IKResult {
            machine_coords,
            metadata,
        })
    }

    #[napi]
    pub fn forward_kinematics_sync(
        &self,
        machine_coords: Vec<MachineAxisCoordinate>,
    ) -> Result<Vec<CartesianPoint>> {
        kinematics::batch_forward_kinematics(&machine_coords, &self.axis_config, &self.tool_params)
            .map_err(|e| Error::from_reason(format!("FK solver failed: {}", e)))
    }

    #[napi]
    pub fn process_pipeline_shared(
        &self,
        env: Env,
        gcode_text: String,
        options: Option<IKOptions>,
    ) -> Result<PipelineResultShared> {
        let ik_opts = options.unwrap_or_default();

        let (blocks, parse_meta) = gcode_parser::parse_gcode(&gcode_text)
            .map_err(|e| Error::from_reason(format!("G-code parse failed: {}", e)))?;

        let cartesian_points: Vec<CartesianPoint> = blocks.iter()
            .map(|b| CartesianPoint {
                x: b.x,
                y: b.y,
                z: b.z,
                a: b.a,
                b: b.b,
                c: b.c,
                feedrate: b.feedrate,
                spindle: b.spindle,
            })
            .collect();

        let (machine_coords, ik_meta) = kinematics::batch_inverse_kinematics(
            &cartesian_points, &self.axis_config, &self.tool_params, ik_opts,
        ).map_err(|e| Error::from_reason(format!("IK solver failed: {}", e)))?;

        create_pipeline_result(env, &cartesian_points, &machine_coords, parse_meta, ik_meta)
    }

    #[napi]
    pub fn process_pipeline_shared_mmap(
        &self,
        env: Env,
        file_path: String,
        options: Option<IKOptions>,
    ) -> Result<PipelineResultShared> {
        let ik_opts = options.unwrap_or_default();

        let (blocks, parse_meta) = gcode_parser::parse_gcode_mmap(&file_path)
            .map_err(|e| Error::from_reason(format!("G-code parse failed: {}", e)))?;

        let cartesian_points: Vec<CartesianPoint> = blocks.iter()
            .map(|b| CartesianPoint {
                x: b.x,
                y: b.y,
                z: b.z,
                a: b.a,
                b: b.b,
                c: b.c,
                feedrate: b.feedrate,
                spindle: b.spindle,
            })
            .collect();

        let (machine_coords, ik_meta) = kinematics::batch_inverse_kinematics(
            &cartesian_points, &self.axis_config, &self.tool_params, ik_opts,
        ).map_err(|e| Error::from_reason(format!("IK solver failed: {}", e)))?;

        create_pipeline_result(env, &cartesian_points, &machine_coords, parse_meta, ik_meta)
    }

    #[napi(getter)]
    pub fn get_machine_config(&self) -> MachineAxisConfig {
        self.axis_config.clone()
    }

    #[napi(getter)]
    pub fn get_tool_params(&self) -> ToolParameters {
        self.tool_params.clone()
    }
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct VoxelStats {
    pub total_voxels: u32,
    pub solid_voxels: u32,
    pub removed_ratio: f64,
    pub voxel_size: f64,
    pub grid_size_x: u32,
    pub grid_size_y: u32,
    pub grid_size_z: u32,
    pub bounds_min_x: f64,
    pub bounds_min_y: f64,
    pub bounds_min_z: f64,
    pub bounds_max_x: f64,
    pub bounds_max_y: f64,
    pub bounds_max_z: f64,
}

#[napi]
pub struct VoxelSimulator {
    engine: VoxelEngine,
}

#[napi]
impl VoxelSimulator {
    #[napi(constructor)]
    pub fn new(
        min_x: f64, min_y: f64, min_z: f64,
        max_x: f64, max_y: f64, max_z: f64,
        voxel_size: f64,
        tool_radius: f64,
        tool_length: f64,
        corner_radius: Option<f64>,
    ) -> Self {
        let bounds = Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z);
        let cr = corner_radius.unwrap_or(0.0);
        let engine = VoxelEngine::new(bounds, voxel_size, tool_radius, tool_length, cr);
        Self { engine }
    }

    #[napi]
    pub fn set_tool(&mut self, radius: f64, length: f64, corner_radius: Option<f64>) {
        self.engine.set_tool(radius, length, corner_radius.unwrap_or(0.0));
    }

    #[napi]
    pub fn set_holder(&mut self, length: f64, radius: f64) {
        self.engine.set_holder(length, radius);
    }

    #[napi]
    pub fn add_fixture(
        &mut self,
        min_x: f64, min_y: f64, min_z: f64,
        max_x: f64, max_y: f64, max_z: f64,
    ) {
        let aabb = Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z);
        self.engine.add_fixture_aabb(aabb);
    }

    #[napi]
    pub fn clear_fixtures(&mut self) {
        self.engine.clear_fixtures();
    }

    #[napi]
    pub fn carve_at(&mut self, tip_x: f64, tip_y: f64, tip_z: f64) -> u32 {
        self.engine.carve_at(tip_x, tip_y, tip_z) as u32
    }

    #[napi]
    pub fn carve_segment(
        &mut self,
        x0: f64, y0: f64, z0: f64,
        x1: f64, y1: f64, z1: f64,
        max_steps: Option<u32>,
    ) -> u32 {
        let steps = max_steps.unwrap_or(256) as usize;
        self.engine.carve_segment(x0, y0, z0, x1, y1, z1, steps) as u32
    }

    #[napi]
    pub fn check_tool_collision(&self, tip_x: f64, tip_y: f64, tip_z: f64) -> bool {
        self.engine.check_tool_collision(tip_x, tip_y, tip_z)
    }

    #[napi]
    pub fn check_fixture_collision(&self, tip_x: f64, tip_y: f64, tip_z: f64) -> bool {
        self.engine.check_fixture_collision(tip_x, tip_y, tip_z)
    }

    #[napi]
    pub fn extract_surface_points(&self, env: Env, max_count: Option<u32>) -> Result<JsArrayBuffer> {
        let max = max_count.unwrap_or(50000) as usize;
        let points = self.engine.extract_surface_points(max);
        let count = points.len();
        let byte_len = count * 3 * std::mem::size_of::<f32>();
        let mut data: Vec<u8> = vec![0u8; byte_len];
        let float_ptr = data.as_mut_ptr() as *mut f32;
        for (i, &(x, y, z)) in points.iter().enumerate() {
            unsafe {
                *float_ptr.add(i * 3) = x;
                *float_ptr.add(i * 3 + 1) = y;
                *float_ptr.add(i * 3 + 2) = z;
            }
        }
        let buf_value = env.create_arraybuffer_with_data(data)?;
        Ok(buf_value.into_raw())
    }

    #[napi]
    pub fn get_stats(&self) -> VoxelStats {
        let (sx, sy, sz) = self.engine.grid_size();
        let (min_x, min_y, min_z) = self.engine.bounds_min();
        let (max_x, max_y, max_z) = self.engine.bounds_max();
        VoxelStats {
            total_voxels: self.engine.total_voxels() as u32,
            solid_voxels: self.engine.solid_voxels() as u32,
            removed_ratio: self.engine.removed_ratio(),
            voxel_size: self.engine.voxel_size(),
            grid_size_x: sx as u32,
            grid_size_y: sy as u32,
            grid_size_z: sz as u32,
            bounds_min_x: min_x,
            bounds_min_y: min_y,
            bounds_min_z: min_z,
            bounds_max_x: max_x,
            bounds_max_y: max_y,
            bounds_max_z: max_z,
        }
    }

    #[napi]
    pub fn reset(&mut self) {
        self.engine.reset_workpiece();
    }

    #[napi]
    pub fn rebuild_octree(&mut self) {
        self.engine.rebuild_octree();
    }
}
