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
    width: f32,                   // width of the grid in pixels
    height: f32,                  // height of the grid in pixels
    x_offset: position::Position, // for positioning the grid on the screen
    y_offset: position::Position, // for positioning the grid on the screen

    width_cells: usize,                     // number of cells
    height_cells: usize,                    // number of cells
    cell_bg_color: macroquad::color::Color, // color of the cells

    gap: f32, // space between cells (in pixels)
    gap_color: macroquad::color::Color,

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
        }
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

    /// # create a grid
    ///
    /// ## problems
    ///
    /// there are a shit ton of feilds and I wanted the new function 
    /// to not have a trillion args.
    /// It is "normal" (more like intended) to create a new Grid and then call a bunch of setters to customize it 
    /// to your liking
    pub fn new(width: f32, height: f32, x_cells: usize, y_cells: usize, gap: f32) -> Self {
        const WIDTH: i32 = 10;
        const HEIGHT: i32 = 10;
        Grid {
            width,
            height,
            width_cells: x_cells,
            height_cells: y_cells,
            cell_bg_color: RED,
            gap,
            gap_color: BLACK,
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
        }
    }

    // returns the (width, height) of each cell
    fn calculate_dimensions(&self) -> (f32, f32) {
        // in pixels
        let total_x_gap_space = (self.width_cells + 1) as f32 * self.gap;
        let total_y_gap_space = (self.height_cells + 1) as f32 * self.gap;

        let cell_width = (self.width - total_x_gap_space as f32) / self.width_cells as f32;
        let cell_height = (self.height - total_y_gap_space as f32) / self.height_cells as f32;

        (cell_width, cell_height)
    }

    /// # draw it!
    /// this does not change any state
    /// your gonna want to put this in the main
    /// loop or something like that
    pub fn draw(&self) {
        // draw background (the gap color)
        let x_offset = position::as_pixels(self.x_offset, self.width, screen_width());
        let y_offset = position::as_pixels(self.y_offset, self.height, screen_height());
        draw_rectangle(x_offset, y_offset, self.width, self.height, self.gap_color);

        // draw cells
        let (cell_width, cell_height) = self.calculate_dimensions();

        for i in 0..self.height_cells {
            for j in 0..self.width_cells {
                self.draw_cell(i, j, cell_width, cell_height, x_offset, y_offset);
            }
        }
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
        if let Some(text) = &self.cells[row][col].text {
            // shifted because read the readme
            let y_pos = y_pos + cell_height;

            // center the text or something idk
            let text_dim = macroquad::text::measure_text(text, None, cell_height as u16, 1.0); // 1.0 is default
            let centered_x = (cell_width - text_dim.width) / 2.0 + x_pos;
            let centered_y = y_pos - (cell_height - text_dim.height) / 2.0;

            draw_text(text, centered_x, centered_y, cell_height, BLACK);
        }
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
        self.cells[row][col].text = t;
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
        if let Some( (row, col) ) = self.get_selected_cell_index() {
            self.set_cell_text(row, col, text);
        }
    }
}
