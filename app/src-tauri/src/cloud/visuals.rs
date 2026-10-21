//! Deterministic Task 1 visuals. The numeric/geometry data is also supplied to reviewers.
use super::{error, generation};
use crate::contracts::Result;
use font8x8::UnicodeFonts;
use serde_json::Value;
const COLORS: [[u8; 3]; 8] = [
    [47, 104, 148],
    [196, 101, 54],
    [78, 139, 99],
    [151, 98, 161],
    [201, 165, 57],
    [96, 141, 151],
    [161, 101, 114],
    [110, 115, 123],
];
struct Canvas {
    w: usize,
    h: usize,
    p: Vec<u8>,
}
impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            p: vec![255; w * h * 3],
        }
    }
    fn pixel(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            let at = (y as usize * self.w + x as usize) * 3;
            self.p[at..at + 3].copy_from_slice(&c);
        }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y.max(0)..(y + h).min(self.h as i32) {
            for xx in x.max(0)..(x + w).min(self.w as i32) {
                self.pixel(xx, yy, c);
            }
        }
    }
    fn line(&mut self, a: (i32, i32), b: (i32, i32), c: [u8; 3]) {
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let steps = dx.abs().max(dy.abs()).max(1);
        for i in 0..=steps {
            self.pixel(a.0 + dx * i / steps, a.1 + dy * i / steps, c);
        }
    }
    fn arrow(&mut self, a: (i32, i32), b: (i32, i32)) {
        self.line(a, b, [70, 70, 70]);
        let angle = ((b.1 - a.1) as f64).atan2((b.0 - a.0) as f64);
        for d in [-0.45, 0.45] {
            self.line(
                b,
                (
                    b.0 - (14. * (angle + d).cos()) as i32,
                    b.1 - (14. * (angle + d).sin()) as i32,
                ),
                [70, 70, 70],
            );
        }
    }
    fn text(&mut self, x: i32, y: i32, s: &str, scale: i32, max_chars: usize) -> Result<()> {
        let value = s
            .replace(['–', '—', '−'], "-")
            .replace(['‘', '’'], "'")
            .replace(['“', '”'], "\"");
        for (i, ch) in value.chars().enumerate() {
            let glyph = font8x8::BASIC_FONTS
                .get(ch)
                .or_else(|| font8x8::LATIN_FONTS.get(ch))
                .ok_or_else(|| error("图表标签含不支持的字符，请使用英文标签和常用数字单位"))?;
            let row = i / max_chars;
            let col = i % max_chars;
            for (yy, bits) in glyph.iter().enumerate() {
                for xx in 0..8 {
                    if bits & (1 << xx) != 0 {
                        self.rect(
                            x + col as i32 * 8 * scale + xx * scale,
                            y + row as i32 * 11 * scale + yy as i32 * scale,
                            scale,
                            scale,
                            [35, 43, 50],
                        );
                    }
                }
            }
        }
        Ok(())
    }
    fn png(self) -> Result<Vec<u8>> {
        let mut out = vec![];
        {
            let mut e = png::Encoder::new(&mut out, self.w as u32, self.h as u32);
            e.set_color(png::ColorType::Rgb);
            e.set_depth(png::BitDepth::Eight);
            let mut w = e.write_header().map_err(error)?;
            w.write_image_data(&self.p).map_err(error)?;
        }
        Ok(out)
    }
}
fn label(v: &Value, key: &str) -> Result<String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.chars().count() <= 120)
        .map(str::to_owned)
        .ok_or_else(|| error(format!("图表缺少或超长 {key}")))
}
pub fn template(kind: &str) -> Result<Value> {
    use serde_json::json;
    Ok(match kind {
        "table" => {
            json!({"kind":"table","title":"Context and units","columns":["Category","2019 (thousands)","2023 (thousands)"],"rows":[["First","10","20"],["Second","30","40"],["Third","50","60"]]})
        }
        "bar" | "line" | "pie" => {
            json!({"kind":kind,"title":"Place and time context","unit":"thousands of people","categories":["First","Second","Third"],"series":[{"name":"2023","values":[10,20,30]}]})
        }
        "process" => {
            json!({"kind":"process","title":"How material is processed","steps":[{"id":"s1","label":"First step"},{"id":"s2","label":"Second step"},{"id":"s3","label":"Third step"}],"connections":[{"from":"s1","to":"s2"},{"from":"s2","to":"s3"}]})
        }
        "map" => {
            json!({"kind":"map","title":"A place before and after development","panels":[{"title":"Before","features":[{"label":"Park","type":"park","x":0.1,"y":0.1,"width":0.3,"height":0.4},{"label":"Road","type":"road","x":0.0,"y":0.6,"width":1.0,"height":0.08}]},{"title":"After","features":[{"label":"Library","type":"building","x":0.1,"y":0.1,"width":0.3,"height":0.4},{"label":"Road","type":"road","x":0.0,"y":0.6,"width":1.0,"height":0.08}]}]})
        }
        "combined" => {
            json!({"kind":"combined","title":"Related changes","panels":[template("bar")?,template("pie")?]})
        }
        _ => return Err(error("不支持此写作图表类型")),
    })
}
pub fn render(v: &Value) -> Result<Vec<u8>> {
    let kind = v["kind"].as_str().unwrap_or("table");
    if kind == "table" {
        return generation::table_png(v);
    }
    if kind == "combined" {
        let panels = v["panels"]
            .as_array()
            .filter(|p| p.len() == 2)
            .ok_or_else(|| error("组合图需要两张数据图表"))?;
        let mut c = Canvas::new(1200, 1520);
        for (i, panel) in panels.iter().enumerate() {
            if !["bar", "line", "pie"].contains(&panel["kind"].as_str().unwrap_or("")) {
                return Err(error("组合图的子图必须是柱图、折线或饼图"));
            }
            let bytes = render(panel)?;
            let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
            let mut reader = decoder.read_info().map_err(error)?;
            let mut image = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut image).map_err(error)?;
            if info.width != 1200 || info.height != 760 {
                return Err(error("组合图尺寸无效"));
            }
            let start = i * 1200 * 760 * 3;
            c.p[start..start + 1200 * 760 * 3].copy_from_slice(&image[..1200 * 760 * 3]);
        }
        return c.png();
    }
    let title = label(v, "title")?;
    let mut c = Canvas::new(1200, 760);
    c.text(36, 24, &title, 2, 70)?;
    match kind {
        "bar" | "line" | "pie" => {
            let unit = label(v, "unit")?;
            c.text(36, 88, &format!("Unit: {unit}"), 2, 70)?;
            let cats = v["categories"]
                .as_array()
                .filter(|a| (2..=8).contains(&a.len()))
                .ok_or_else(|| error("图表需要2至8个类别"))?;
            let series = v["series"]
                .as_array()
                .filter(|a| (1..=3).contains(&a.len()))
                .ok_or_else(|| error("图表需要1至3个数据系列"))?;
            let mut data = vec![];
            for s in series {
                let name = label(s, "name")?;
                let values = s["values"]
                    .as_array()
                    .filter(|a| a.len() == cats.len())
                    .ok_or_else(|| error("图表类别与数据数量不一致"))?;
                let mut numbers = vec![];
                for x in values {
                    numbers.push(
                        x.as_f64()
                            .filter(|n| n.is_finite() && *n >= 0. && *n <= 1e12)
                            .ok_or_else(|| error("图表数值必须是有限的非负数"))?,
                    );
                }
                data.push((name, numbers));
            }
            let labels: Vec<_> = cats
                .iter()
                .map(|s| {
                    s.as_str()
                        .filter(|s| !s.trim().is_empty() && s.chars().count() <= 40)
                        .ok_or_else(|| error("图表类别标签无效"))
                })
                .collect::<Result<_>>()?;
            if kind == "pie" {
                if data.len() != 1 {
                    return Err(error("一张饼图只接受一个数据系列"));
                }
                let sum: f64 = data[0].1.iter().sum();
                if sum <= 0. {
                    return Err(error("饼图总量必须大于0"));
                }
                let mut ends = vec![];
                let mut total = 0.;
                for n in &data[0].1 {
                    total += n / sum * std::f64::consts::TAU;
                    ends.push(total);
                }
                for y in 165..650 {
                    for x in 75..560 {
                        let dx = x as f64 - 315.;
                        let dy = y as f64 - 410.;
                        if dx * dx + dy * dy <= 220. * 220. {
                            let angle = (dy.atan2(dx) + std::f64::consts::FRAC_PI_2)
                                .rem_euclid(std::f64::consts::TAU);
                            let index = ends
                                .iter()
                                .position(|e| angle <= *e)
                                .unwrap_or(ends.len() - 1);
                            c.pixel(x, y, COLORS[index]);
                        }
                    }
                }
                for (i, name) in labels.iter().enumerate() {
                    let y = 165 + i as i32 * 63;
                    c.rect(605, y, 24, 24, COLORS[i]);
                    c.text(
                        647,
                        y,
                        &format!(
                            "{name}: {} ({:.1}%)",
                            data[0].1[i],
                            data[0].1[i] / sum * 100.
                        ),
                        2,
                        31,
                    )?;
                }
                c.text(80, 700, &format!("{}; total = {sum}", data[0].0), 2, 65)?;
            } else {
                let peak = data
                    .iter()
                    .flat_map(|(_, a)| a.iter())
                    .fold(0_f64, |a, b| a.max(*b));
                let top = if peak == 0. { 1. } else { peak * 1.1 };
                let left = 120;
                let right = 1140;
                let bottom = 600;
                let height = 420;
                for tick in 0..=5 {
                    let y = bottom - height * tick / 5;
                    c.line((left, y), (right, y), [215, 219, 222]);
                    c.text(16, y - 7, &format!("{:.2}", top * tick as f64 / 5.), 1, 13)?;
                }
                c.line((left, 180), (left, bottom), [40, 40, 40]);
                c.line((left, bottom), (right, bottom), [40, 40, 40]);
                let span = (right - left) as f64 / labels.len() as f64;
                for (i, name) in labels.iter().enumerate() {
                    c.text(
                        left + (span * i as f64) as i32 + 4,
                        bottom + 16,
                        name,
                        1,
                        14,
                    )?;
                }
                for (si, (name, values)) in data.iter().enumerate() {
                    c.rect(120 + si as i32 * 330, 710, 18, 18, COLORS[si]);
                    c.text(145 + si as i32 * 330, 710, name, 1, 35)?;
                    let mut prev: Option<(i32, i32)> = None;
                    for (i, value) in values.iter().enumerate() {
                        let y = bottom - (*value / top * height as f64) as i32;
                        let x = left + (span * (i as f64 + 0.5)) as i32;
                        if kind == "bar" {
                            let width = (span * 0.72 / data.len() as f64) as i32;
                            let bx =
                                left + (span * i as f64 + span * 0.14) as i32 + si as i32 * width;
                            c.rect(bx, y, (width - 3).max(1), bottom - y, COLORS[si]);
                            c.text(bx, y - 16, &value.to_string(), 1, 12)?;
                        } else {
                            if let Some(p) = prev {
                                for dy in -1..=1 {
                                    c.line((p.0, p.1 + dy), (x, y + dy), COLORS[si]);
                                }
                            }
                            c.rect(x - 4, y - 4, 9, 9, COLORS[si]);
                            c.text(x + 7, y - 14, &value.to_string(), 1, 12)?;
                            prev = Some((x, y));
                        }
                    }
                }
            }
        }
        "process" => {
            let steps = v["steps"]
                .as_array()
                .filter(|a| (3..=9).contains(&a.len()))
                .ok_or_else(|| error("流程图需要3至9个步骤"))?;
            let mut boxes = std::collections::HashMap::new();
            for (i, s) in steps.iter().enumerate() {
                let id = label(s, "id")?;
                let x = 70 + (i % 3) as i32 * 390;
                let y = 180 + (i / 3) as i32 * 175;
                if boxes.insert(id, (x, y)).is_some() {
                    return Err(error("流程步骤 ID 重复"));
                }
            }
            let edges = v["connections"]
                .as_array()
                .filter(|a| !a.is_empty() && a.len() <= 20)
                .ok_or_else(|| error("流程缺少有效连接"))?;
            for edge in edges {
                let a = boxes
                    .get(&label(edge, "from")?)
                    .ok_or_else(|| error("流程起点不存在"))?;
                let b = boxes
                    .get(&label(edge, "to")?)
                    .ok_or_else(|| error("流程终点不存在"))?;
                if a.1 == b.1 {
                    if a.0 < b.0 {
                        c.arrow((a.0 + 290, a.1 + 55), (b.0 - 4, b.1 + 55));
                    } else {
                        c.arrow((a.0, a.1 + 55), (b.0 + 294, b.1 + 55));
                    }
                } else {
                    c.arrow((a.0 + 145, a.1 + 110), (b.0 + 145, b.1 - 4));
                }
            }
            for step in steps {
                let (x, y) = boxes[&label(step, "id")?];
                c.rect(x, y, 290, 110, [223, 235, 241]);
                let text = label(step, "label")?;
                let scale = if text.chars().count() <= 64 { 2 } else { 1 };
                c.text(
                    x + 12,
                    y + 18,
                    &text,
                    scale,
                    if scale == 2 { 16 } else { 32 },
                )?;
            }
        }
        "map" => {
            let panels = v["panels"]
                .as_array()
                .filter(|a| (1..=2).contains(&a.len()))
                .ok_or_else(|| error("地图需要1至2个时期或方案"))?;
            for (pi, panel) in panels.iter().enumerate() {
                let ox = 40 + pi as i32 * 590;
                c.text(ox, 135, &label(panel, "title")?, 2, 32)?;
                c.rect(ox, 200, 530, 450, [237, 238, 223]);
                let features = panel["features"]
                    .as_array()
                    .filter(|a| !a.is_empty() && a.len() <= 30)
                    .ok_or_else(|| error("地图缺少有效地物"))?;
                for f in features {
                    let number = |key: &str| {
                        f[key]
                            .as_f64()
                            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
                            .ok_or_else(|| error("地图坐标须在0至1之间"))
                    };
                    let x = number("x")?;
                    let y = number("y")?;
                    let w = number("width")?;
                    let h = number("height")?;
                    if w <= 0. || h <= 0. || x + w > 1. || y + h > 1. {
                        return Err(error("地图地物超出绘制范围"));
                    }
                    let color = match f["type"].as_str() {
                        Some("park") => [169, 202, 147],
                        Some("water") => [143, 191, 213],
                        Some("road") => [177, 179, 180],
                        Some("building") => [211, 179, 141],
                        _ => return Err(error("地图地物类型无效")),
                    };
                    let px = ox + (x * 530.) as i32;
                    let py = 200 + (y * 450.) as i32;
                    c.rect(px, py, (w * 530.) as i32, (h * 450.) as i32, color);
                    c.text(
                        px + 4,
                        py + 5,
                        &label(f, "label")?,
                        1,
                        ((w * 530.) as usize / 8).max(4),
                    )?;
                }
                c.arrow((ox + 485, 190), (ox + 485, 160));
                c.text(ox + 478, 140, "N", 1, 2)?;
            }
        }
        _ => return Err(error("不支持此 Task1 图表类型")),
    }
    c.png()
}
