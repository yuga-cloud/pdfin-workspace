#[derive(Debug, Clone)]
pub struct PdfWord {
    pub text: String,

    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,

    pub center_x: f32,
    pub center_y: f32,
}

impl PdfWord {
    pub fn new(text: String, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            text,
            left,
            right,
            top,
            bottom,
            center_x: (left + right) / 2.0,
            center_y: (top + bottom) / 2.0,
        }
    }

    pub fn height(&self) -> f32 {
        (self.bottom - self.top).abs()
    }
}

#[derive(Debug, Clone)]
pub struct PdfRow {
    pub words: Vec<PdfWord>,

    pub top: f32,
    pub bottom: f32,
}

impl PdfRow {
    pub fn height(&self) -> f32 {
        (self.bottom - self.top).max(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn column_count(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(0)
    }
}
