use macroquad::prelude::*;

mod cell;
mod position;

pub use position::Position;

/// # the point of this crate!
/// used to represent and draw a grid to the screen
/// heres the repo: https://github.com/TheDinner22/macroquad_grid
///
/// ## construction
/// use the new method or the default method
///
/// ## notes
///
/// only has private feilds so you interface with it via
/// methods (mainly getters and setters)
///
/// ## stuff you can do
///
/// - creating a grid
/// - selecting a cell
/// - changing selected cells color
/// - changing default cell bg color
/// - changing gap color
/// - changing grids postion with Position enum
/// - setting color of a specific cell
/// - writing text to a specific cell
/// - writing text to the selected cell
/// - getting the selected cell's index
/// - drawing the grid
pub struct Grid {
    pub width: f32,               // width of the grid in pixels
    pub height: f32,              // height of the grid in pixels
    x_offset: position::Position, // for positioning the grid on the screen
    y_offset: position::Position, // for positioning the grid on the screen

    pub auto_resize_text: bool,
    width_cells: usize,                     // number of cells
    height_cells: usize,                    // number of cells
    cell_bg_color: macroquad::color::Color, // color of the cells

    pub gap: f32, // space between cells (in pixels)
    pub gap_color: macroquad::color::Color,

    // is a vec really needed here? how use const bro
    cells: Vec<Vec<cell::Cell>>,

    selected_cell: Option<(usize, usize)>, // selected cell (if needed)
    selected_color: Option<macroquad::color::Color>,
}

impl Default for Grid {
    fn default() -> Self {
        const WIDTH: usize = 10;
        const HEIGHT: usize = 10;
        Grid {
            width: screen_width(),
            height: screen_height(),
            width_cells: WIDTH,
            height_cells: HEIGHT,
            cell_bg_color: RED,
            gap: 3.0,
            gap_color: PINK,
            selected_cell: None,
            selected_color: Some(BLUE),
            // ignore the HORRID line below this comment
            // it just makes a 2D list of cell::default's
            // there are HEIGHT inner lists and they all have WIDTH elements
            cells: (0..HEIGHT)
                .into_iter()
                .map(|_| {
                    (0..WIDTH)
                        .into_iter()
                        .map(|_| cell::Cell::default())
                        .collect::<Vec<_>>()
                })
                .collect(),
            x_offset: position::Position::default(),
            y_offset: position::Position::default(),
            auto_resize_text: true,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Dimensions {
    pub rows: usize,
    pub cols: usize,
}

impl Dimensions {
    pub fn max(self, other: Self) -> Self {
        Self {
            rows: self.rows.max(other.rows),
            cols: self.cols.max(other.cols),
        }
    }
    pub fn from_wh(width: usize, height: usize) -> Self {
        Self {
            rows: height,
            cols: width,
        }
    }
    pub fn width(&self) -> usize {
        self.cols
    }
    pub fn height(&self) -> usize {
        self.rows
    }
}

impl Grid {
    /// position the grid somewhere on the screen
    pub fn set_x_offset(&mut self, x_offset: position::Position) {
        self.x_offset = x_offset;
    }

    /// position the grid somewhere on the screen
    pub fn set_y_offset(&mut self, y_offset: position::Position) {
        self.y_offset = y_offset;
    }

    pub fn resize(&mut self, width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) {
        let width = width.into().unwrap_or(self.width_cells);
        let height = height.into().unwrap_or(self.height_cells);
        if Dimensions::from_wh(width, height) != self.dimensions() {
            self.cells
                .resize_with(height, || vec![Default::default(); width]);
            for row in self.cells.iter_mut() {
                row.resize_with(width, Default::default);
            }
            self.width_cells = width;
            self.height_cells = height;
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions::from_wh(self.width_cells, self.height_cells)
    }

    pub fn rows(&self) -> usize {
        self.height_cells
    }
    pub fn cols(&self) -> usize {
        self.width_cells
    }
    pub fn height(&self) -> usize {
        self.height_cells
    }
    pub fn width(&self) -> usize {
        self.width_cells
    }

    /// # create a grid
    ///
    /// ## problems
    ///
    /// there are a shit ton of feilds and I wanted the new function
    /// to not have a trillion args.
    /// It is "normal" (more like intended) to create a new Grid and then call a bunch of setters to customize it
    /// to your liking
    pub fn new(width: f32, height: f32, x_cells: usize, y_cells: usize, gap: f32) -> Self {
        Grid {
            width,
            height,
            width_cells: x_cells,
            height_cells: y_cells,
            cell_bg_color: WHITE,
            gap,
            gap_color: BLACK,
            selected_cell: None,
            selected_color: Some(BLUE),
            // ignore the HORRID line below this comment
            // it just makes a 2D list of cell::default's
            // there are HEIGHT inner lists and they all have WIDTH elements
            cells: (0..y_cells)
                .into_iter()
                .map(|_| {
                    (0..x_cells)
                        .into_iter()
                        .map(|_| cell::Cell::default())
                        .collect::<Vec<_>>()
                })
                .collect(),
            x_offset: position::Position::default(),
            y_offset: position::Position::default(),
            auto_resize_text: true,
        }
    }

    // returns the (width, height) of each cell
    fn calculate_dimensions(&self) -> (f32, f32) {
        // in pixels
        let total_x_gap_space = (self.width_cells + 1) as f32 * self.gap;
        let total_y_gap_space = (self.height_cells + 1) as f32 * self.gap;

        let cell_width = (self.width - total_x_gap_space as f32).max(0.0) / self.width_cells as f32;
        let cell_height =
            (self.height - total_y_gap_space as f32).max(0.0) / self.height_cells as f32;

        (cell_width, cell_height)
    }

    /// # draw it!
    /// this does not change any state
    /// your gonna want to put this in the main
    /// loop or something like that
    pub fn draw(&self) {
        // draw background (the gap color)
        let x_offset = self.x_offset.as_pixels(self.width, screen_width());
        let y_offset = self.y_offset.as_pixels(self.height, screen_height());
        draw_rectangle(x_offset, y_offset, self.width, self.height, self.gap_color);

        // draw cells
        let (cell_width, cell_height) = self.calculate_dimensions();

        for row in 0..self.height_cells {
            for col in 0..self.width_cells {
                self.draw_cell(row, col, cell_width, cell_height, x_offset, y_offset);
            }
        }
        // draw_rectangle_lines(x_offset, y_offset, self.width, self.height, 1.0, BLACK);
    }

    // only called from the double for loop in the draw function
    // this way it does not look crouded as fuck
    //
    // this function calculates the cells position (takes gap into account)
    // it also handles any special coloring that might need to happen
    // it also prints any text to the screen (if applicable)
    fn draw_cell(
        &self,
        row: usize,
        col: usize,
        cell_width: f32,
        cell_height: f32,
        x_offset: f32,
        y_offset: f32,
    ) {
        // cell cords
        let x_pos = x_offset + self.gap + col as f32 * (cell_width + self.gap as f32);
        let y_pos = y_offset + self.gap + row as f32 * (cell_height + self.gap as f32);

        // cell color
        let mut color = self.cell_bg_color;
        // if this is the selected_cell, use the other color
        if let Some((selected_row, selected_col)) = self.selected_cell {
            if selected_row == row && selected_col == col {
                color = self
                    .selected_color
                    .expect("there was a selected cell but no selected color");
            }
        }
        // and if it had a preset color then use that
        else if let Some(set_color) = self.cells[row][col].color {
            // somehow we never reach this??
            color = set_color;
        }

        // draw it!
        draw_rectangle(x_pos, y_pos, cell_width, cell_height, color);

        // draw the text if this cell has any
        let text = &self.cells[row][col].text;
        if !text.is_empty() {
            // shifted because read the readme
            let y_pos = y_pos + cell_height;

            // center the text or something idk
            let mut cell_height = cell_height;
            let mut text = text.as_str();
            loop {
                let text_dim = macroquad::text::measure_text(text, None, cell_height as u16, 1.0); // 1.0 is default
                if self.auto_resize_text && text_dim.width > cell_width {
                    cell_height *= cell_width / text_dim.width * 0.9;
                    continue;
                } else if text_dim.width > cell_width {
                    let char_count = text.chars().count();
                    let mut it = text.chars();
                    for _ in 0..(char_count as f32 * cell_width / text_dim.width) as usize {
                        it.next();
                    }
                    text = &text[..text.len() - it.as_str().len()];
                    continue;
                }
                let centered_x = (cell_width - text_dim.width) / 2.0 + x_pos;
                let centered_y = y_pos - (cell_height - text_dim.height) / 2.0;

                draw_text(text, centered_x, centered_y, cell_height, BLACK);
                break;
            }
        }
        // draw_rectangle_lines(x_pos, y_pos, cell_width, cell_height, 1.0, BLACK);
    }

    pub fn select_from_mouse(&mut self) -> Option<(usize, usize)> {
        let result = self.mouse_hovered_cell();
        self.select_cell(result);
        result
    }

    pub fn mouse_hovered_cell(&mut self) -> Option<(usize, usize)> {
        self.translate_click(mouse_position().into())
    }

    pub fn translate_click(&self, pos: Vec2) -> Option<(usize, usize)> {
        let x_offset = position::as_pixels(self.x_offset, self.width, screen_width());
        let y_offset = position::as_pixels(self.y_offset, self.height, screen_height());

        let rect = Rect {
            x: x_offset,
            y: y_offset,
            w: self.width,
            h: self.height,
        };
        if !rect.contains(pos) {
            return None;
        }
        let (cell_width, cell_height) = self.calculate_dimensions();
        // for row in 0..self.height_cells {
        //     for col in 0..self.width_cells {
        //         self.draw_cell(row, col, cell_width, cell_height, x_offset, y_offset);
        //     }
        // }
        let col = (pos.x - (x_offset + self.gap)) / (cell_width + self.gap as f32);
        let row = (pos.y - (y_offset + self.gap)) / (cell_height + self.gap as f32);
        Some((row as usize, col as usize))
    }

    /// # select a cell
    ///
    /// ## warning
    /// if the selected cell is out of bounds
    /// it might lead to a panic later
    pub fn select_cell(&mut self, cell_index: Option<(usize, usize)>) {
        self.selected_cell = cell_index;
    }

    /// returns the (row, col) index of the selected cell
    pub fn get_selected_cell_index(&self) -> Option<(usize, usize)> {
        self.selected_cell
    }

    /// changes the default bg color of the given cell
    ///
    /// ## panics
    /// if the row or col is out of bounds indexing into the 2D vector
    /// which represents the grid (its private u can't see it)
    pub fn color_cell(&mut self, row: usize, col: usize, color: macroquad::color::Color) {
        self.cells[row][col].color = Some(color);
    }

    /// # sets default bg color for all cells
    ///
    /// different from color_cell becuase this one applies to all
    /// uncolored and unselected cells
    /// this function panics
    pub fn set_cell_bg_color(&mut self, color: macroquad::color::Color) {
        self.cell_bg_color = color;
    }

    /// color the gap between cells
    pub fn set_gap_color(&mut self, color: macroquad::color::Color) {
        self.gap_color = color;
    }

    /// when selected, a cell will have this color
    pub fn set_selected_cell_color(&mut self, color: macroquad::color::Color) {
        self.selected_color = Some(color);
    }

    /// # write text to a cell
    ///
    /// ## panics
    /// if row and col are out of bounds
    ///
    /// ## generic option
    /// so the text arg is the text to be written
    /// - if the Option is None, there will be no text
    /// - if the Option is Some(text), I call text.to_string()
    /// and then write the resulting String to the screen
    pub fn set_cell_text<T>(&mut self, row: usize, col: usize, text: Option<T>)
    where
        T: ToString,
    {
        // map value to string
        let t = text.map(|val| val.to_string());
        // set value
        self.cells[row][col].text = t.unwrap_or_default();
    }

    pub fn cell_text_mut(&mut self, row: usize, col: usize) -> &mut String {
        &mut self.cells[row][col].text
    }

    /// same as set_cell_text
    /// but instead of providing a row and col
    /// it just writes the text onto the selected cell
    ///
    /// ## no selected cell
    ///
    /// if there is no selected cell, this
    /// method does nothing
    ///
    /// ## panics
    ///
    /// if the selected cell happens to be out of bounds,
    /// this function panics
    pub fn set_selected_cell_text<T>(&mut self, text: Option<T>)
    where
        T: ToString,
    {
        // only do something if there is a selected cell
        if let Some((row, col)) = self.get_selected_cell_index() {
            self.set_cell_text(row, col, text);
        }
    }
}
