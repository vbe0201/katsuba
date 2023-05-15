use std::io::{self, BufWriter, Stdout, Write};

const CELL: u8 = b'=';
const EMPTY: u8 = b' ';

/// A user-visible progress bar which prints to stdout.
pub struct ProgressBar<const WIDTH: usize> {
    bar: [u8; WIDTH],
    out: BufWriter<Stdout>,
    cells: u32,
    total: u32,
}

impl<const WIDTH: usize> ProgressBar<WIDTH> {
    pub fn new(prefix: &str, total: u32) -> io::Result<Self> {
        let mut this = Self {
            bar: [EMPTY; WIDTH],
            out: BufWriter::new(io::stdout()),
            cells: 0,
            total,
        };

        println!("{prefix}");
        this.render(0)?;

        Ok(this)
    }

    #[inline]
    pub fn update(&mut self, current: u32) -> io::Result<()> {
        let cells = (current * WIDTH as u32)
            .checked_div(self.total)
            .unwrap_or(0);

        if cells > self.cells {
            // Update the buffered progress bar with more filled cells.
            self.bar[self.cells as usize..cells as usize].fill(CELL);
            self.cells = cells;

            // Display it to screen.
            self.render(current)?;
        }

        Ok(())
    }

    #[inline]
    fn render(&mut self, current: u32) -> io::Result<()> {
        // Write the buffered progress bar to stdout.
        write!(self.out, "\r[")?;
        self.out.write_all(&self.bar)?;
        write!(self.out, "] {}/{} ", current, self.total)?;

        // Flush to display the changes.
        self.out.flush()
    }
}
