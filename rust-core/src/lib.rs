#![deny(clippy::all)]
#![allow(clippy::too_many_arguments)]

use napi::*;
use napi_derive::napi;

pub mod types;
pub mod gcode_parser;
pub mod kinematics;
pub mod shared_memory;

use types::*;
use shared_memory::*;

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
