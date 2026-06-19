use std::f64::consts::PI;

const VOXEL_STATE_EMPTY: u8 = 0;
const VOXEL_STATE_SOLID: u8 = 1;

#[derive(Debug, Clone)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    pub fn from_center_size(cx: f64, cy: f64, cz: f64, sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            min_x: cx - sx * 0.5,
            min_y: cy - sy * 0.5,
            min_z: cz - sz * 0.5,
            max_x: cx + sx * 0.5,
            max_y: cy + sy * 0.5,
            max_z: cz + sz * 0.5,
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
            && self.min_z <= other.max_z
            && self.max_z >= other.min_z
    }

    pub fn contains_point(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x <= self.max_x
            && y >= self.min_y
            && y <= self.max_y
            && z >= self.min_z
            && z <= self.max_z
    }

    pub fn size_x(&self) -> f64 { self.max_x - self.min_x }
    pub fn size_y(&self) -> f64 { self.max_y - self.min_y }
    pub fn size_z(&self) -> f64 { self.max_z - self.min_z }
}

#[derive(Debug, Clone)]
pub struct CylinderTool {
    pub radius: f64,
    pub length: f64,
    pub corner_radius: f64,
}

impl CylinderTool {
    pub fn new(radius: f64, length: f64, corner_radius: f64) -> Self {
        let cr = corner_radius.min(radius).min(length * 0.5).max(0.0);
        Self { radius, length, corner_radius: cr }
    }

    pub fn aabb_at(&self, tip_x: f64, tip_y: f64, tip_z: f64) -> Aabb {
        Aabb::new(
            tip_x - self.radius,
            tip_y,
            tip_z - self.radius,
            tip_x + self.radius,
            tip_y + self.length,
            tip_z + self.radius,
        )
    }

    pub fn distance_sq_at_xy(&self, tip_x: f64, tip_z: f64, px: f64, pz: f64) -> f64 {
        let dx = px - tip_x;
        let dz = pz - tip_z;
        dx * dx + dz * dz
    }

    pub fn contains_point(&self, tip_x: f64, tip_y: f64, tip_z: f64, px: f64, py: f64, pz: f64) -> bool {
        let r_sq = self.radius * self.radius;
        let dist_sq = self.distance_sq_at_xy(tip_x, tip_z, px, pz);
        if dist_sq > r_sq + 1e-6 {
            return false;
        }

        let y_bottom = tip_y;
        let y_top = tip_y + self.length;
        let cr = self.corner_radius;

        if cr <= 0.0 {
            return py >= y_bottom && py <= y_top;
        }

        let cyl_y_bottom = y_bottom + cr;
        let cyl_y_top = y_top - cr;

        if py >= cyl_y_bottom && py <= cyl_y_top {
            return true;
        }

        if py > y_top || py < y_bottom {
            return false;
        }

        let r_eff = if py < cyl_y_bottom {
            let dy = cyl_y_bottom - py;
            let h = cr - dy;
            if h <= 0.0 { return false; }
            let r_edge = (cr * cr - h * h).max(0.0).sqrt();
            self.radius - cr + r_edge
        } else {
            let dy = py - cyl_y_top;
            let h = cr - dy;
            if h <= 0.0 { return false; }
            let r_edge = (cr * cr - h * h).max(0.0).sqrt();
            self.radius - cr + r_edge
        };

        dist_sq <= r_eff * r_eff + 1e-6
    }

    pub fn holder_aabb_at(&self, tip_x: f64, tip_y: f64, tip_z: f64, holder_len: f64, holder_r: f64) -> Aabb {
        let holder_bottom = tip_y + self.length;
        let holder_top = holder_bottom + holder_len;
        let r = holder_r.max(self.radius);
        Aabb::new(
            tip_x - r,
            holder_bottom,
            tip_z - r,
            tip_x + r,
            holder_top,
            tip_z + r,
        )
    }
}

pub struct VoxelGrid {
    pub bounds: Aabb,
    pub voxel_size: f64,
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub data: Vec<u8>,
    pub surface_voxel_count: usize,
}

impl VoxelGrid {
    pub fn new(bounds: Aabb, voxel_size: f64) -> Self {
        let vs = voxel_size.max(0.01);
        let size_x = (bounds.size_x() / vs).ceil().max(1.0) as usize;
        let size_y = (bounds.size_y() / vs).ceil().max(1.0) as usize;
        let size_z = (bounds.size_z() / vs).ceil().max(1.0) as usize;
        let total = size_x * size_y * size_z;
        let data = vec![VOXEL_STATE_SOLID; total];
        Self {
            bounds,
            voxel_size: vs,
            size_x,
            size_y,
            size_z,
            data,
            surface_voxel_count: 0,
        }
    }

    pub fn total_voxels(&self) -> usize {
        self.size_x * self.size_y * self.size_z
    }

    pub fn solid_voxel_count(&self) -> usize {
        self.data.iter().filter(|&&v| v != VOXEL_STATE_EMPTY).count()
    }

    #[inline]
    pub fn idx(&self, ix: usize, iy: usize, iz: usize) -> usize {
        ix + iy * self.size_x + iz * self.size_x * self.size_y
    }

    pub fn world_to_voxel(&self, x: f64, y: f64, z: f64) -> (isize, isize, isize) {
        let ix = ((x - self.bounds.min_x) / self.voxel_size).floor() as isize;
        let iy = ((y - self.bounds.min_y) / self.voxel_size).floor() as isize;
        let iz = ((z - self.bounds.min_z) / self.voxel_size).floor() as isize;
        (ix, iy, iz)
    }

    pub fn voxel_to_world_center(&self, ix: usize, iy: usize, iz: usize) -> (f64, f64, f64) {
        let cx = self.bounds.min_x + (ix as f64 + 0.5) * self.voxel_size;
        let cy = self.bounds.min_y + (iy as f64 + 0.5) * self.voxel_size;
        let cz = self.bounds.min_z + (iz as f64 + 0.5) * self.voxel_size;
        (cx, cy, cz)
    }

    pub fn get_voxel(&self, ix: isize, iy: isize, iz: isize) -> u8 {
        if ix < 0 || iy < 0 || iz < 0 {
            return VOXEL_STATE_EMPTY;
        }
        if ix as usize >= self.size_x || iy as usize >= self.size_y || iz as usize >= self.size_z {
            return VOXEL_STATE_EMPTY;
        }
        self.data[self.idx(ix as usize, iy as usize, iz as usize)]
    }

    pub fn set_voxel(&mut self, ix: isize, iy: isize, iz: isize, value: u8) {
        if ix < 0 || iy < 0 || iz < 0 { return; }
        if ix as usize >= self.size_x || iy as usize >= self.size_y || iz as usize >= self.size_z {
            return;
        }
        let idx = self.idx(ix as usize, iy as usize, iz as usize);
        self.data[idx] = value;
    }

    pub fn is_surface_voxel(&self, ix: usize, iy: usize, iz: usize) -> bool {
        if self.data[self.idx(ix, iy, iz)] == VOXEL_STATE_EMPTY {
            return false;
        }
        if ix == 0 || iy == 0 || iz == 0 {
            return true;
        }
        if ix + 1 >= self.size_x || iy + 1 >= self.size_y || iz + 1 >= self.size_z {
            return true;
        }
        let ixi = ix as isize;
        let iyi = iy as isize;
        let izi = iz as isize;
        self.get_voxel(ixi - 1, iyi, izi) == VOXEL_STATE_EMPTY
            || self.get_voxel(ixi + 1, iyi, izi) == VOXEL_STATE_EMPTY
            || self.get_voxel(ixi, iyi - 1, izi) == VOXEL_STATE_EMPTY
            || self.get_voxel(ixi, iyi + 1, izi) == VOXEL_STATE_EMPTY
            || self.get_voxel(ixi, iyi, izi - 1) == VOXEL_STATE_EMPTY
            || self.get_voxel(ixi, iyi, izi + 1) == VOXEL_STATE_EMPTY
    }

    pub fn extract_surface_voxels(&self, max_count: usize) -> Vec<(f32, f32, f32)> {
        let mut result = Vec::with_capacity(max_count.min(self.total_voxels()));
        let half = (self.voxel_size * 0.5) as f32;
        for iz in 0..self.size_z {
            for iy in 0..self.size_y {
                for ix in 0..self.size_x {
                    if self.data[self.idx(ix, iy, iz)] == VOXEL_STATE_EMPTY {
                        continue;
                    }
                    if !self.is_surface_voxel(ix, iy, iz) {
                        continue;
                    }
                    let (cx, cy, cz) = self.voxel_to_world_center(ix, iy, iz);
                    result.push((cx as f32, cy as f32, cz as f32));
                    if result.len() >= max_count {
                        return result;
                    }
                }
            }
        }
        result
    }

    pub fn carve_cylinder_tool(&mut self, tool: &CylinderTool, tip_x: f64, tip_y: f64, tip_z: f64) -> usize {
        let tool_aabb = tool.aabb_at(tip_x, tip_y, tip_z);
        let (min_ix, min_iy, min_iz) = self.world_to_voxel(tool_aabb.min_x, tool_aabb.min_y, tool_aabb.min_z);
        let (max_ix, max_iy, max_iz) = self.world_to_voxel(tool_aabb.max_x, tool_aabb.max_y, tool_aabb.max_z);

        let min_ix = min_ix.max(0) as usize;
        let max_ix = (max_ix as usize).min(self.size_x.saturating_sub(1));
        let min_iy = min_iy.max(0) as usize;
        let max_iy = (max_iy as usize).min(self.size_y.saturating_sub(1));
        let min_iz = min_iz.max(0) as usize;
        let max_iz = (max_iz as usize).min(self.size_z.saturating_sub(1));

        if min_ix > max_ix || min_iy > max_iy || min_iz > max_iz {
            return 0;
        }

        let half_voxel = self.voxel_size * 0.5;
        let mut removed = 0usize;

        for iz in min_iz..=max_iz {
            for iy in min_iy..=max_iy {
                for ix in min_ix..=max_ix {
                    let idx = self.idx(ix, iy, iz);
                    if self.data[idx] == VOXEL_STATE_EMPTY {
                        continue;
                    }
                    let (cx, cy, cz) = self.voxel_to_world_center(ix, iy, iz);
                    if self._voxel_intersects_tool(tool, tip_x, tip_y, tip_z, cx, cy, cz, half_voxel) {
                        self.data[idx] = VOXEL_STATE_EMPTY;
                        removed += 1;
                    }
                }
            }
        }
        removed
    }

    fn _voxel_intersects_tool(
        &self,
        tool: &CylinderTool,
        tip_x: f64, tip_y: f64, tip_z: f64,
        cx: f64, cy: f64, cz: f64,
        half_v: f64,
    ) -> bool {
        if tool.contains_point(tip_x, tip_y, tip_z, cx, cy, cz) {
            return true;
        }
        let corners = [
            (cx - half_v, cy - half_v, cz - half_v),
            (cx + half_v, cy - half_v, cz - half_v),
            (cx - half_v, cy + half_v, cz - half_v),
            (cx + half_v, cy + half_v, cz - half_v),
            (cx - half_v, cy - half_v, cz + half_v),
            (cx + half_v, cy - half_v, cz + half_v),
            (cx - half_v, cy + half_v, cz + half_v),
            (cx + half_v, cy + half_v, cz + half_v),
        ];
        let mut inside = 0;
        for &(px, py, pz) in &corners {
            if tool.contains_point(tip_x, tip_y, tip_z, px, py, pz) {
                inside += 1;
            }
        }
        if inside > 0 && inside < 8 {
            return true;
        }
        let dx = cx - tip_x;
        let dz = cz - tip_z;
        let dist_xz = (dx * dx + dz * dz).sqrt();
        if dist_xz > tool.radius + half_v {
            return false;
        }
        if cy + half_v < tip_y || cy - half_v > tip_y + tool.length {
            return false;
        }
        true
    }

    pub fn carve_tool_path_segment(
        &mut self,
        tool: &CylinderTool,
        start_x: f64, start_y: f64, start_z: f64,
        end_x: f64, end_y: f64, end_z: f64,
        max_steps: usize,
    ) -> usize {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let dz = end_z - start_z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 1e-6 {
            return self.carve_cylinder_tool(tool, end_x, end_y, end_z);
        }
        let step_size = self.voxel_size * 0.5;
        let steps = ((dist / step_size).ceil() as usize).max(1).min(max_steps.max(1));
        let mut removed = 0usize;
        for i in 0..=steps {
            let t = if steps == 0 { 0.0 } else { i as f64 / steps as f64 };
            let x = start_x + dx * t;
            let y = start_y + dy * t;
            let z = start_z + dz * t;
            removed += self.carve_cylinder_tool(tool, x, y, z);
        }
        removed
    }

    pub fn check_collision_aabb(&self, aabb: &Aabb) -> bool {
        let (min_ix, min_iy, min_iz) = self.world_to_voxel(aabb.min_x, aabb.min_y, aabb.min_z);
        let (max_ix, max_iy, max_iz) = self.world_to_voxel(aabb.max_x, aabb.max_y, aabb.max_z);

        let min_ix = min_ix.max(0) as usize;
        let max_ix = (max_ix as usize).min(self.size_x.saturating_sub(1));
        let min_iy = min_iy.max(0) as usize;
        let max_iy = (max_iy as usize).min(self.size_y.saturating_sub(1));
        let min_iz = min_iz.max(0) as usize;
        let max_iz = (max_iz as usize).min(self.size_z.saturating_sub(1));

        if min_ix > max_ix || min_iy > max_iy || min_iz > max_iz {
            return false;
        }

        for iz in min_iz..=max_iz {
            for iy in min_iy..=max_iy {
                for ix in min_ix..=max_ix {
                    if self.data[self.idx(ix, iy, iz)] != VOXEL_STATE_EMPTY {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn reset(&mut self) {
        for v in self.data.iter_mut() {
            *v = VOXEL_STATE_SOLID;
        }
        self.surface_voxel_count = 0;
    }

    pub fn volume_removed_ratio(&self) -> f64 {
        let total = self.total_voxels() as f64;
        if total <= 0.0 { return 0.0; }
        let empty = self.data.iter().filter(|&&v| v == VOXEL_STATE_EMPTY).count() as f64;
        empty / total
    }
}

pub struct OctreeNode {
    pub bounds: Aabb,
    pub children: Option<[Box<OctreeNode>; 8]>,
    pub is_leaf: bool,
    pub has_solid: bool,
    pub depth: u32,
}

impl OctreeNode {
    pub fn build(bounds: Aabb, depth: u32, max_depth: u32, grid: &VoxelGrid) -> Self {
        let has_solid = Self::check_any_solid(&bounds, grid);
        if depth >= max_depth || !has_solid {
            return Self {
                bounds,
                children: None,
                is_leaf: true,
                has_solid,
                depth,
            };
        }

        let hx = bounds.size_x() * 0.5;
        let hy = bounds.size_y() * 0.5;
        let hz = bounds.size_z() * 0.5;
        let cx = (bounds.min_x + bounds.max_x) * 0.5;
        let cy = (bounds.min_y + bounds.max_y) * 0.5;
        let cz = (bounds.min_z + bounds.max_z) * 0.5;

        let mut children: [Box<OctreeNode>; 8] = std::array::from_fn(|i| {
            let ox = if i & 1 == 0 { 0.0 } else { hx };
            let oy = if i & 2 == 0 { 0.0 } else { hy };
            let oz = if i & 4 == 0 { 0.0 } else { hz };
            let child_bounds = Aabb::new(
                bounds.min_x + ox,
                bounds.min_y + oy,
                bounds.min_z + oz,
                cx + ox,
                cy + oy,
                cz + oz,
            );
            Box::new(Self::build(child_bounds, depth + 1, max_depth, grid))
        });

        Self {
            bounds,
            children: Some(children),
            is_leaf: false,
            has_solid,
            depth,
        }
    }

    fn check_any_solid(bounds: &Aabb, grid: &VoxelGrid) -> bool {
        let (min_ix, min_iy, min_iz) = grid.world_to_voxel(bounds.min_x, bounds.min_y, bounds.min_z);
        let (max_ix, max_iy, max_iz) = grid.world_to_voxel(bounds.max_x, bounds.max_y, bounds.max_z);

        let min_ix = min_ix.max(0) as usize;
        let max_ix = (max_ix as usize).min(grid.size_x.saturating_sub(1));
        let min_iy = min_iy.max(0) as usize;
        let max_iy = (max_iy as usize).min(grid.size_y.saturating_sub(1));
        let min_iz = min_iz.max(0) as usize;
        let max_iz = (max_iz as usize).min(grid.size_z.saturating_sub(1));

        if min_ix > max_ix || min_iy > max_iy || min_iz > max_iz {
            return false;
        }

        for iz in min_iz..=max_iz {
            for iy in min_iy..=max_iy {
                for ix in min_ix..=max_ix {
                    if grid.data[grid.idx(ix, iy, iz)] != VOXEL_STATE_EMPTY {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn query_collision(&self, aabb: &Aabb) -> bool {
        if !self.bounds.intersects(aabb) {
            return false;
        }
        if !self.has_solid {
            return false;
        }
        if self.is_leaf {
            return self.has_solid;
        }
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.query_collision(aabb) {
                    return true;
                }
            }
        }
        false
    }
}

pub struct VoxelEngine {
    pub grid: VoxelGrid,
    pub tool: CylinderTool,
    pub octree: Option<OctreeNode>,
    pub max_octree_depth: u32,
    pub holder_length: f64,
    pub holder_radius: f64,
    pub fixture_aabbs: Vec<Aabb>,
}

impl VoxelEngine {
    pub fn new(bounds: Aabb, voxel_size: f64, tool_radius: f64, tool_length: f64, corner_radius: f64) -> Self {
        let grid = VoxelGrid::new(bounds, voxel_size);
        Self {
            grid,
            tool: CylinderTool::new(tool_radius, tool_length, corner_radius),
            octree: None,
            max_octree_depth: 4,
            holder_length: 80.0,
            holder_radius: 35.0,
            fixture_aabbs: Vec::new(),
        }
    }

    pub fn set_tool(&mut self, radius: f64, length: f64, corner_radius: f64) {
        self.tool = CylinderTool::new(radius, length, corner_radius);
    }

    pub fn set_holder(&mut self, length: f64, radius: f64) {
        self.holder_length = length;
        self.holder_radius = radius;
    }

    pub fn add_fixture_aabb(&mut self, aabb: Aabb) {
        self.fixture_aabbs.push(aabb);
    }

    pub fn clear_fixtures(&mut self) {
        self.fixture_aabbs.clear();
    }

    pub fn rebuild_octree(&mut self) {
        self.octree = Some(OctreeNode::build(
            self.grid.bounds.clone(),
            0,
            self.max_octree_depth,
            &self.grid,
        ));
    }

    pub fn carve_at(&mut self, tip_x: f64, tip_y: f64, tip_z: f64) -> usize {
        self.grid.carve_cylinder_tool(&self.tool, tip_x, tip_y, tip_z)
    }

    pub fn carve_segment(
        &mut self,
        x0: f64, y0: f64, z0: f64,
        x1: f64, y1: f64, z1: f64,
        max_steps: usize,
    ) -> usize {
        self.grid.carve_tool_path_segment(&self.tool, x0, y0, z0, x1, y1, z1, max_steps)
    }

    pub fn check_tool_collision(&self, tip_x: f64, tip_y: f64, tip_z: f64) -> bool {
        let holder_aabb = self.tool.holder_aabb_at(
            tip_x, tip_y, tip_z,
            self.holder_length,
            self.holder_radius,
        );
        if let Some(octree) = &self.octree {
            octree.query_collision(&holder_aabb)
        } else {
            self.grid.check_collision_aabb(&holder_aabb)
        }
    }

    pub fn check_fixture_collision(&self, tip_x: f64, tip_y: f64, tip_z: f64) -> bool {
        let holder_aabb = self.tool.holder_aabb_at(
            tip_x, tip_y, tip_z,
            self.holder_length,
            self.holder_radius,
        );
        for fixture in &self.fixture_aabbs {
            if fixture.intersects(&holder_aabb) {
                return true;
            }
        }
        false
    }

    pub fn extract_surface_points(&self, max_count: usize) -> Vec<(f32, f32, f32)> {
        self.grid.extract_surface_voxels(max_count)
    }

    pub fn reset_workpiece(&mut self) {
        self.grid.reset();
        self.octree = None;
    }

    pub fn total_voxels(&self) -> usize {
        self.grid.total_voxels()
    }

    pub fn solid_voxels(&self) -> usize {
        self.grid.solid_voxel_count()
    }

    pub fn removed_ratio(&self) -> f64 {
        self.grid.volume_removed_ratio()
    }

    pub fn voxel_size(&self) -> f64 {
        self.grid.voxel_size
    }

    pub fn bounds_min(&self) -> (f64, f64, f64) {
        (self.grid.bounds.min_x, self.grid.bounds.min_y, self.grid.bounds.min_z)
    }

    pub fn bounds_max(&self) -> (f64, f64, f64) {
        (self.grid.bounds.max_x, self.grid.bounds.max_y, self.grid.bounds.max_z)
    }

    pub fn grid_size(&self) -> (usize, usize, usize) {
        (self.grid.size_x, self.grid.size_y, self.grid.size_z)
    }

    pub fn raw_voxel_data_ptr(&self) -> *const u8 {
        self.grid.data.as_ptr()
    }

    pub fn raw_voxel_data_slice(&self) -> &[u8] {
        &self.grid.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_grid_creation() {
        let bounds = Aabb::new(-50.0, 0.0, -50.0, 50.0, 100.0, 50.0);
        let grid = VoxelGrid::new(bounds, 5.0);
        assert_eq!(grid.size_x, 20);
        assert_eq!(grid.size_y, 20);
        assert_eq!(grid.size_z, 20);
        assert_eq!(grid.total_voxels(), 8000);
        assert_eq!(grid.solid_voxel_count(), 8000);
    }

    #[test]
    fn test_cylinder_tool_contains() {
        let tool = CylinderTool::new(5.0, 20.0, 0.0);
        assert!(tool.contains_point(0.0, 0.0, 0.0, 0.0, 10.0, 0.0));
        assert!(tool.contains_point(0.0, 0.0, 0.0, 4.9, 5.0, 0.0));
        assert!(!tool.contains_point(0.0, 0.0, 0.0, 6.0, 5.0, 0.0));
        assert!(!tool.contains_point(0.0, 0.0, 0.0, 0.0, -1.0, 0.0));
        assert!(!tool.contains_point(0.0, 0.0, 0.0, 0.0, 21.0, 0.0));
    }

    #[test]
    fn test_carve_tool() {
        let bounds = Aabb::new(-20.0, -20.0, -20.0, 20.0, 20.0, 20.0);
        let mut grid = VoxelGrid::new(bounds, 2.0);
        let tool = CylinderTool::new(5.0, 30.0, 0.0);
        let removed = grid.carve_cylinder_tool(&tool, 0.0, -20.0, 0.0);
        assert!(removed > 0);
        assert!(grid.solid_voxel_count() < grid.total_voxels());
    }

    #[test]
    fn test_aabb_intersects() {
        let a = Aabb::new(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        let b = Aabb::new(5.0, 5.0, 5.0, 15.0, 15.0, 15.0);
        assert!(a.intersects(&b));
        let c = Aabb::new(20.0, 20.0, 20.0, 30.0, 30.0, 30.0);
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_check_collision_aabb() {
        let bounds = Aabb::new(-50.0, 0.0, -50.0, 50.0, 100.0, 50.0);
        let grid = VoxelGrid::new(bounds, 5.0);
        let test_aabb = Aabb::new(-10.0, 20.0, -10.0, 10.0, 40.0, 10.0);
        assert!(grid.check_collision_aabb(&test_aabb));
        let outside = Aabb::new(100.0, 200.0, 100.0, 200.0, 300.0, 200.0);
        assert!(!grid.check_collision_aabb(&outside));
    }
}
