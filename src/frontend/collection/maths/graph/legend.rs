use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4, vec2};

#[derive(Debug, Clone)]
pub struct PlotLegendEntry {
    pub label: String,
    pub color: Vec4,
}

impl PlotLegendEntry {
    pub fn new(label: impl Into<String>, color: Vec4) -> Self {
        Self {
            label: label.into(),
            color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlotLegend {
    pub entries: Vec<PlotLegendEntry>,
    pub font_height: f32,
    pub line_length: f32,
    pub line_thickness: f32,
    pub row_height: f32,
    pub text_color: Vec4,
}

impl PlotLegend {
    pub fn new(entries: impl Into<Vec<PlotLegendEntry>>) -> Self {
        Self {
            entries: entries.into(),
            font_height: 0.16,
            line_length: 0.42,
            line_thickness: 0.035,
            row_height: 0.28,
            text_color: Vec4::ONE,
        }
    }

    pub fn with_font_height(mut self, font_height: f32) -> Self {
        self.font_height = font_height;
        self
    }

    pub fn with_line_style(mut self, line_length: f32, line_thickness: f32) -> Self {
        self.line_length = line_length;
        self.line_thickness = line_thickness;
        self
    }

    pub fn with_row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height;
        self
    }

    pub fn with_text_color(mut self, text_color: Vec4) -> Self {
        self.text_color = text_color;
        self
    }
}

impl Project for PlotLegend {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for (index, entry) in self.entries.iter().enumerate() {
            let y = -(index as f32) * self.row_height;

            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(0.0, y, 0.0),
                end: Vec3::new(self.line_length, y, 0.0),
                thickness: self.line_thickness,
                color: entry.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
            ctx.emit(RenderPrimitive::Text {
                content: entry.label.clone(),
                height: self.font_height,
                color: self.text_color,
                font_name: None,
                offset: Vec3::new(self.line_length + 0.18, y - self.font_height * 0.38, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for PlotLegend {
    fn local_bounds(&self) -> Bounds {
        let height = if self.entries.is_empty() {
            self.font_height
        } else {
            self.row_height * self.entries.len() as f32
        };

        Bounds::new(
            vec2(0.0, -height),
            vec2(self.line_length + 2.2, self.font_height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn plot_legend_keeps_entries() {
        let legend = PlotLegend::new(vec![PlotLegendEntry::new("series", Vec4::ONE)]);

        assert_eq!(legend.entries.len(), 1);
        assert_eq!(legend.entries[0].label, "series");
    }
}
