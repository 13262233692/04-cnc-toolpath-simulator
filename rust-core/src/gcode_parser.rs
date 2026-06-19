use crate::types::*;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use memmap2::Mmap;
use ahash::AHashMap;

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
}

impl<'a> Lexer<'a> {
    #[inline(always)]
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line_number: 1,
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
            c
        } else {
            0
        }
    }

    #[inline(always)]
    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            let c = self.peek();
            match c {
                b' ' | b'\t' | b'\r' => {
                    self.advance();
                }
                b'(' => {
                    self.advance();
                    while self.pos < self.input.len() && self.peek() != b')' {
                        if self.peek() == b'\n' {
                            self.line_number += 1;
                        }
                        self.advance();
                    }
                    if self.peek() == b')' {
                        self.advance();
                    }
                }
                b';' => {
                    while self.pos < self.input.len() && self.peek() != b'\n' {
                        self.advance();
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
        while self.pos < self.input.len() {
            let c = self.peek();
            if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' {
                self.advance();
            } else if (c == b'-' || c == b'+') && (self.pos > start) {
                let prev = unsafe { *self.input.get_unchecked(self.pos - 1) };
                if prev == b'e' || prev == b'E' {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let slice = unsafe { std::slice::from_raw_parts(self.input.as_ptr().add(start), self.pos - start) };
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
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        int_part = int_part.wrapping_mul(10).wrapping_add((bytes[i] - b'0') as u64);
        i += 1;
    }

    let mut frac_part: u64 = 0;
    let mut frac_divisor: f64 = 1.0;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            frac_part = frac_part.wrapping_mul(10).wrapping_add((bytes[i] - b'0') as u64);
            frac_divisor *= 10.0;
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
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp = exp * 10 + (bytes[i] - b'0') as i32;
            i += 1;
        }
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

    let mut hm: AHashMap<u8, u8> = AHashMap::new();
    for c in b"ABCXYZFGSTNM".iter() {
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
            b'X' => {
                current_block.x = value;
            }
            b'Y' => {
                current_block.y = value;
            }
            b'Z' => {
                current_block.z = value;
            }
            b'A' => {
                current_block.a = value;
            }
            b'B' => {
                current_block.b = value;
            }
            b'C' => {
                current_block.c = value;
            }
            b'F' => {
                current_block.feedrate = value;
            }
            b'S' => {
                current_block.spindle = value;
            }
            b'T' => {
                current_block.tool_number = value as u32;
            }
            b'N' => {
                current_block.block_number = value as u32;
            }
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

    if block.a.is_nan() {
        block.a = 0.0;
    }
    if block.b.is_nan() {
        block.b = 0.0;
    }
    if block.c.is_nan() {
        block.c = 0.0;
    }
    if block.feedrate.is_nan() {
        block.feedrate = 0.0;
    }
    if block.spindle.is_nan() {
        block.spindle = 0.0;
    }

    if !block.is_motion && block.g_code == 0 {
        block.g_code = *modal_g;
        match *modal_g {
            0 => {
                block.is_motion = true;
                block.is_rapid = true;
            }
            1 | 2 | 3 => {
                block.is_motion = true;
                block.is_rapid = false;
            }
            _ => {}
        }
    }

    if block.is_motion {
        meta.motion_blocks += 1;
        if block.is_rapid {
            meta.rapid_blocks += 1;
        }

        if block.x < meta.min_x {
            meta.min_x = block.x;
        }
        if block.x > meta.max_x {
            meta.max_x = block.x;
        }
        if block.y < meta.min_y {
            meta.min_y = block.y;
        }
        if block.y > meta.max_y {
            meta.max_y = block.y;
        }
        if block.z < meta.min_z {
            meta.min_z = block.z;
        }
        if block.z > meta.max_z {
            meta.max_z = block.z;
        }

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

    blocks.push(*block);
}

pub fn parse_gcode_mmap(file_path: &str) -> Result<(Vec<GCodeBlock>, ParseMetadata), String> {
    let file = File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mmap = unsafe { Mmap::map(&file).map_err(|e| format!("内存映射失败: {}", e))? };
    let content = unsafe { std::str::from_utf8_unchecked(&mmap) };
    parse_gcode(content)
}

#[allow(dead_code)]
pub fn parse_gcode_chunked(input: &str, chunk_size: usize) -> Result<(Vec<GCodeBlock>, ParseMetadata), String> {
    let _ = chunk_size;
    parse_gcode(input)
}

pub fn read_file_to_string(file_path: &str) -> Result<String, String> {
    let mut file = File::open(file_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let metadata = file.metadata().map_err(|e| format!("获取文件元数据失败: {}", e))?;
    let mut content = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut content).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(content)
}
