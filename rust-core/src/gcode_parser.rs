use crate::types::*;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use memmap2::Mmap;
use ahash::AHashMap;

const MAX_LINE_CHARS: usize = 8192;
const MAX_COMMENT_CHARS: usize = 4096;
const MAX_NUMBER_DIGITS: usize = 64;
const MAX_ARC_DISCRETIZED_POINTS: usize = 4096;
const DEFAULT_ARC_TOLERANCE: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Word,
    Number,
    Comment,
    Newline,
    Eof,
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    line_number: u32,
    line_char_count: usize,
}

impl<'a> Lexer<'a> {
    #[inline(always)]
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line_number: 1,
            line_char_count: 0,
        }
    }

    #[inline(always)]
    fn peek(&self) -> u8 {
        if self.pos < self.input.len() {
            unsafe { *self.input.get_unchecked(self.pos) }
        } else {
            0
        }
    }

    #[inline(always)]
    fn advance(&mut self) -> u8 {
        if self.pos < self.input.len() {
            let c = unsafe { *self.input.get_unchecked(self.pos) };
            self.pos += 1;
            self.line_char_count += 1;
            c
        } else {
            0
        }
    }

    #[inline(always)]
    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            if self.line_char_count >= MAX_LINE_CHARS {
                while self.pos < self.input.len() && self.peek() != b'\n' {
                    self.pos += 1;
                }
                if self.peek() == b'\n' {
                    self.pos += 1;
                    self.line_number += 1;
                    self.line_char_count = 0;
                }
                continue;
            }

            let c = self.peek();
            match c {
                b' ' | b'\t' | b'\r' => {
                    self.advance();
                }
                b'(' => {
                    self.advance();
                    let mut counter: usize = 0;
                    while self.pos < self.input.len() && self.peek() != b')' {
                        if self.peek() == b'\n' {
                            self.line_number += 1;
                            self.line_char_count = 0;
                        }
                        self.advance();
                        counter += 1;
                        if counter >= MAX_COMMENT_CHARS {
                            break;
                        }
                    }
                    if self.peek() == b')' {
                        self.advance();
                    }
                }
                b';' => {
                    let mut counter: usize = 0;
                    while self.pos < self.input.len() && self.peek() != b'\n' {
                        self.advance();
                        counter += 1;
                        if counter >= MAX_COMMENT_CHARS {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    #[inline(always)]
    fn parse_number(&mut self) -> f64 {
        let start = self.pos;
        if self.peek() == b'-' || self.peek() == b'+' {
            self.advance();
        }
        let mut digits: usize = 0;
        while self.pos < self.input.len() && digits < MAX_NUMBER_DIGITS {
            let c = self.peek();
            if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' {
                self.advance();
                digits += 1;
            } else if (c == b'-' || c == b'+') && (self.pos > start) {
                let prev = unsafe { *self.input.get_unchecked(self.pos - 1) };
                if prev == b'e' || prev == b'E' {
                    self.advance();
                    digits += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let len = (self.pos - start).min(MAX_NUMBER_DIGITS);
        let slice = unsafe { std::slice::from_raw_parts(self.input.as_ptr().add(start), len) };
        let s = unsafe { std::str::from_utf8_unchecked(slice) };
        fast_float_parse(s)
    }
}

#[inline(always)]
fn fast_float_parse(s: &str) -> f64 {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }

    let mut i = 0;
    let mut negative = false;

    if bytes[i] == b'-' {
        negative = true;
        i += 1;
    } else if bytes[i] == b'+' {
        i += 1;
    }

    let mut int_part: u64 = 0;
    let mut int_digits: u32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() && int_digits < 19 {
        int_part = int_part.wrapping_mul(10).wrapping_add((bytes[i] - b'0') as u64);
        int_digits += 1;
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    let mut frac_part: u64 = 0;
    let mut frac_divisor: f64 = 1.0;
    let mut frac_digits: u32 = 0;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() && frac_digits < 15 {
            frac_part = frac_part.wrapping_mul(10).wrapping_add((bytes[i] - b'0') as u64);
            frac_divisor *= 10.0;
            frac_digits += 1;
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    let mut exp: i32 = 0;
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        let mut exp_neg = false;
        if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
            exp_neg = bytes[i] == b'-';
            i += 1;
        }
        let mut exp_digits: u32 = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() && exp_digits < 4 {
            exp = exp.saturating_mul(10).saturating_add((bytes[i] - b'0') as i32);
            exp_digits += 1;
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        exp = exp.clamp(-308, 308);
        if exp_neg {
            exp = -exp;
        }
    }

    let mut result = int_part as f64 + (frac_part as f64 / frac_divisor);
    if exp != 0 {
        result *= 10.0f64.powi(exp);
    }
    if negative {
        result = -result;
    }
    if result.is_nan() || result.is_infinite() {
        0.0
    } else {
        result
    }
}

fn discretize_arc(
    start: &GCodeBlock,
    end: &GCodeBlock,
    clockwise: bool,
    tolerance: f64,
) -> Vec<(f64, f64, f64)> {
    let mut result = Vec::new();
    let x0 = start.x;
    let y0 = start.y;
    let z0 = start.z;
    let x1 = end.x;
    let y1 = end.y;
    let z1 = end.z;

    let i = if end.i.is_nan() { 0.0 } else { end.i };
    let j = if end.j.is_nan() { 0.0 } else { end.j };
    let k = if end.k.is_nan() { 0.0 } else { end.k };
    let r = if end.radius.is_nan() { 0.0 } else { end.radius };

    let (cx, cy, cz, radius) = if r.abs() > 1e-9 {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dz = z1 - z0;
        let chord = (dx * dx + dy * dy + dz * dz).sqrt();
        if chord < 1e-9 || r.abs() * 2.0 < chord {
            return result;
        }
        let h_sq = r * r - (chord / 2.0) * (chord / 2.0);
        let h = if h_sq > 0.0 { h_sq.sqrt() } else { 0.0 };
        let sign = if clockwise { -1.0 } else { 1.0 } * if r > 0.0 { 1.0 } else { -1.0 };
        let mx = (x0 + x1) / 2.0;
        let my = (y0 + y1) / 2.0;
        let mz = (z0 + z1) / 2.0;
        if dz.abs() < 1e-9 {
            let nx = -dy / chord;
            let ny = dx / chord;
            (mx + nx * h * sign, my + ny * h * sign, mz, r.abs())
        } else {
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, (z0 + z1) / 2.0, r.abs())
        }
    } else {
        (x0 + i, y0 + j, z0 + k, (i * i + j * j + k * k).sqrt())
    };

    if radius < 1e-9 {
        return result;
    }

    let start_angle = (y0 - cy).atan2(x0 - cx);
    let mut end_angle = (y1 - cy).atan2(x1 - cx);

    let mut sweep = end_angle - start_angle;
    if clockwise {
        while sweep > 0.0 { sweep -= 2.0 * std::f64::consts::PI; }
        while sweep < -2.0 * std::f64::consts::PI { sweep += 2.0 * std::f64::consts::PI; }
    } else {
        while sweep < 0.0 { sweep += 2.0 * std::f64::consts::PI; }
        while sweep > 2.0 * std::f64::consts::PI { sweep -= 2.0 * std::f64::consts::PI; }
    }

    let chord_len = 2.0 * radius * (sweep.abs() / 2.0).sin();
    let step_angle = if tolerance > 0.0 {
        let max_chord_error = tolerance.min(radius * 0.1);
        2.0 * (1.0 - max_chord_error / radius).acos()
    } else {
        0.01
    };
    let step_angle = step_angle.max(0.0005).min(std::f64::consts::PI / 8.0);

    let num_steps = (sweep.abs() / step_angle).ceil() as usize;
    let num_steps = num_steps.max(2).min(MAX_ARC_DISCRETIZED_POINTS);
    let actual_step = sweep / (num_steps as f64);

    let z_step = if num_steps > 1 { (z1 - z0) / (num_steps as f64) } else { 0.0 };

    for s in 1..num_steps {
        let t = s as f64 * actual_step;
        let angle = start_angle + t;
        let px = cx + radius * angle.cos();
        let py = cy + radius * angle.sin();
        let pz = z0 + z_step * s as f64;
        result.push((px, py, pz));
    }
    result
}

pub fn parse_gcode(input: &str) -> Result<(Vec<GCodeBlock>, ParseMetadata), String> {
    let start = Instant::now();
    let bytes = input.as_bytes();
    let mut lexer = Lexer::new(bytes);

    let mut blocks: Vec<GCodeBlock> = Vec::with_capacity(1024 * 64);
    let mut current_block = GCodeBlock::default();
    let mut metadata = ParseMetadata::default();

    let mut prev_x = 0.0f64;
    let mut prev_y = 0.0f64;
    let mut prev_z = 0.0f64;
    let mut modal_g = 0u32;
    let mut block_number = 0u32;
    let mut in_block = false;
    let mut last_motion_block: Option<GCodeBlock> = None;

    let mut hm: AHashMap<u8, u8> = AHashMap::new();
    for c in b"ABCXYZFGSTNMRIJK".iter() {
        hm.insert(*c, *c);
        hm.insert(c.to_ascii_lowercase(), *c);
    }

    loop {
        lexer.skip_whitespace_and_comments();
        let c = lexer.peek();

        if c == 0 {
            break;
        }

        if c == b'\n' {
            lexer.advance();
            lexer.line_number += 1;
            lexer.line_char_count = 0;
            if in_block {
                finalize_block(
                    &mut current_block,
                    &mut blocks,
                    &mut metadata,
                    &mut prev_x,
                    &mut prev_y,
                    &mut prev_z,
                    &mut modal_g,
                    block_number,
                    &mut last_motion_block,
                );
                block_number += 1;
                in_block = false;
                current_block = GCodeBlock::default();
            }
            continue;
        }

        let letter = if let Some(&l) = hm.get(&c) {
            lexer.advance();
            l
        } else {
            lexer.advance();
            continue;
        };

        in_block = true;
        current_block.line_number = lexer.line_number;
        let value = lexer.parse_number();

        match letter {
            b'G' => {
                let gcode = value as u32;
                current_block.g_code = gcode;
                match gcode {
                    0 | 1 | 2 | 3 => {
                        modal_g = gcode;
                        current_block.g_code = gcode;
                        current_block.is_motion = true;
                        current_block.is_rapid = gcode == 0;
                    }
                    _ => {}
                }
            }
            b'X' => { current_block.x = value; }
            b'Y' => { current_block.y = value; }
            b'Z' => { current_block.z = value; }
            b'A' => { current_block.a = value; }
            b'B' => { current_block.b = value; }
            b'C' => { current_block.c = value; }
            b'I' => { current_block.i = value; }
            b'J' => { current_block.j = value; }
            b'K' => { current_block.k = value; }
            b'R' => { current_block.radius = value; }
            b'F' => { current_block.feedrate = value; }
            b'S' => { current_block.spindle = value; }
            b'T' => { current_block.tool_number = value as u32; }
            b'N' => { current_block.block_number = value as u32; }
            b'M' => {}
            _ => {}
        }
    }

    if in_block {
        finalize_block(
            &mut current_block,
            &mut blocks,
            &mut metadata,
            &mut prev_x,
            &mut prev_y,
            &mut prev_z,
            &mut modal_g,
            block_number,
            &mut last_motion_block,
        );
    }

    metadata.total_lines = lexer.line_number;
    metadata.total_blocks = blocks.len() as u32;
    metadata.parse_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok((blocks, metadata))
}

#[inline(always)]
fn finalize_block(
    block: &mut GCodeBlock,
    blocks: &mut Vec<GCodeBlock>,
    meta: &mut ParseMetadata,
    prev_x: &mut f64,
    prev_y: &mut f64,
    prev_z: &mut f64,
    modal_g: &mut u32,
    _block_num: u32,
    last_motion: &mut Option<GCodeBlock>,
) {
    if block.x.is_nan() {
        block.x = *prev_x;
    } else {
        *prev_x = block.x;
    }
    if block.y.is_nan() {
        block.y = *prev_y;
    } else {
        *prev_y = block.y;
    }
    if block.z.is_nan() {
        block.z = *prev_z;
    } else {
        *prev_z = block.z;
    }

    if block.a.is_nan() { block.a = 0.0; }
    if block.b.is_nan() { block.b = 0.0; }
    if block.c.is_nan() { block.c = 0.0; }
    if block.feedrate.is_nan() { block.feedrate = 0.0; }
    if block.spindle.is_nan() { block.spindle = 0.0; }
    if block.i.is_nan() { block.i = 0.0; }
    if block.j.is_nan() { block.j = 0.0; }
    if block.k.is_nan() { block.k = 0.0; }
    if block.radius.is_nan() { block.radius = 0.0; }

    if !block.is_motion && block.g_code == 0 {
        block.g_code = *modal_g;
        match *modal_g {
            0 => { block.is_motion = true; block.is_rapid = true; }
            1 | 2 | 3 => { block.is_motion = true; block.is_rapid = false; }
            _ => {}
        }
    }

    if block.is_motion {
        if block.g_code == 2 || block.g_code == 3 {
            if let Some(prev) = last_motion {
                let clockwise = block.g_code == 2;
                let arc_points = discretize_arc(prev, block, clockwise, DEFAULT_ARC_TOLERANCE);
                for (px, py, pz) in arc_points.iter() {
                    let mut interpolated = GCodeBlock::default();
                    interpolated.line_number = block.line_number;
                    interpolated.g_code = 1;
                    interpolated.is_motion = true;
                    interpolated.is_rapid = false;
                    interpolated.x = *px;
                    interpolated.y = *py;
                    interpolated.z = *pz;
                    interpolated.a = block.a;
                    interpolated.b = block.b;
                    interpolated.c = block.c;
                    interpolated.feedrate = block.feedrate;
                    interpolated.spindle = block.spindle;
                    accumulate_motion(&interpolated, blocks, meta);
                    blocks.push(interpolated);
                }
            }
        }
        accumulate_motion(block, blocks, meta);
        *last_motion = Some(*block);
    }

    blocks.push(*block);
}

#[inline(always)]
fn accumulate_motion(block: &GCodeBlock, blocks: &[GCodeBlock], meta: &mut ParseMetadata) {
    meta.motion_blocks += 1;
    if block.is_rapid { meta.rapid_blocks += 1; }

    if block.x < meta.min_x { meta.min_x = block.x; }
    if block.x > meta.max_x { meta.max_x = block.x; }
    if block.y < meta.min_y { meta.min_y = block.y; }
    if block.y > meta.max_y { meta.max_y = block.y; }
    if block.z < meta.min_z { meta.min_z = block.z; }
    if block.z > meta.max_z { meta.max_z = block.z; }

    if blocks.len() > 0 {
        let prev = &blocks[blocks.len() - 1];
        let dx = block.x - prev.x;
        let dy = block.y - prev.y;
        let dz = block.z - prev.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        meta.total_distance += dist;

        if !block.is_rapid && block.feedrate > 0.0 {
            meta.estimated_time += dist / (block.feedrate / 60.0);
        } else if block.is_rapid {
            meta.estimated_time += dist / 1000.0;
        }
    }
}

pub fn parse_gcode_mmap(file_path: &str) -> Result<(Vec<GCodeBlock>, ParseMetadata), String> {
    let file = File::open(file_path).map_err(|e| format!("open failed: {}", e))?;
    let mmap = unsafe { Mmap::map(&file).map_err(|e| format!("mmap failed: {}", e))? };
    let content = unsafe { std::str::from_utf8_unchecked(&mmap) };
    parse_gcode(content)
}

#[allow(dead_code)]
pub fn parse_gcode_chunked(input: &str, chunk_size: usize) -> Result<(Vec<GCodeBlock>, ParseMetadata), String> {
    let _ = chunk_size;
    parse_gcode(input)
}

pub fn read_file_to_string(file_path: &str) -> Result<String, String> {
    let mut file = File::open(file_path).map_err(|e| format!("open failed: {}", e))?;
    let metadata = file.metadata().map_err(|e| format!("stat failed: {}", e))?;
    let mut content = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut content).map_err(|e| format!("read failed: {}", e))?;
    Ok(content)
}
