// so a grid is composed of cells?
// this is becuase if we want to write to the grid
// it would be nice if it remembered what it was doing

// simple ass struct, doesn't even have no impl
#[derive(Default, Debug)]
pub struct Cell {
    pub color: Option<macroquad::color::Color>,
    pub text: Option<String>,
    pub text_color: Option<macroquad::color::Color>
}

