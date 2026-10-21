//! Offline comparison against independently labelled reference samples. No model calls.
use crate::contracts::{DesktopError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Sample {
    id: String,
    reference_band: f64,
    predicted_band: f64,
    model: String,
    protocol: String,
    reference_source: String,
}
fn error(s: &str) -> DesktopError {
    DesktopError::new("calibration", s)
}
pub fn measure(input: &Value) -> Result<Value> {
    let samples:Vec<Sample>=serde_json::from_value(input.clone()).map_err(|_|error("基准记录须包含 id、referenceBand、predictedBand、model、protocol、referenceSource；分数为0至9的半分档"))?;
    if samples.is_empty() || samples.len() > 10000 {
        return Err(error("基准文件需要1至10000条记录"));
    }
    type People = BTreeMap<String, (f64, Vec<f64>)>;
    let mut groups: BTreeMap<(String, String), People> = BTreeMap::new();
    for s in samples {
        if [&s.id, &s.model, &s.protocol, &s.reference_source]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 500)
            || [s.reference_band, s.predicted_band]
                .iter()
                .any(|n| !n.is_finite() || !(0.0..=9.0).contains(n) || (n * 2.).fract() != 0.)
        {
            return Err(error("基准标识、来源或分数无效；请使用独立人工参考评分"));
        }
        let person = groups
            .entry((s.model, s.protocol))
            .or_default()
            .entry(s.id)
            .or_insert((s.reference_band, vec![]));
        if person.0 != s.reference_band {
            return Err(error("同一基准样本的参考分不一致，请先完成人工复核"));
        }
        person.1.push(s.predicted_band);
    }
    let mut output = vec![];
    for ((model, protocol), people) in groups {
        let n = people.len();
        let mut mae = 0.;
        let mut half = 0.;
        let mut max_error: f64 = 0.;
        let mut repeat = vec![];
        let mut runs = 0;
        for (reference, values) in people.values() {
            runs += values.len();
            mae += values.iter().map(|v| (v - reference).abs()).sum::<f64>() / values.len() as f64;
            half += values
                .iter()
                .filter(|v| (**v - reference).abs() <= 0.5)
                .count() as f64
                / values.len() as f64;
            for v in values {
                max_error = max_error.max((v - reference).abs());
            }
            if values.len() > 1 {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                repeat.push(
                    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64)
                        .sqrt(),
                );
            }
        }
        mae /= n as f64;
        half /= n as f64;
        let repeat_sd = if repeat.is_empty() {
            None
        } else {
            Some(repeat.iter().sum::<f64>() / repeat.len() as f64)
        };
        output.push(json!({"model":model,"protocol":protocol,"uniqueSamples":n,"runs":runs,"meanAbsoluteError":mae,"withinHalfBand":half,"maxAbsoluteError":max_error,"repeatedSamples":repeat.len(),"repeatStandardDeviation":repeat_sd,"targetMet":n>=100&&mae<=0.5&&half>=0.8}));
    }
    Ok(
        json!({"version":1,"groups":output,"notice":"仅统计上传参考集；参考标注应独立于模型生成，须与开发/调参样本分开。达到统计目标不代表官方考官认证或对所有人群准确。"}),
    )
}
