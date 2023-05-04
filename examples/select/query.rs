use promkit::{build::Builder, crossterm::style, select, Result};

fn main() -> Result<()> {
    loop {
        let mut p = select::Builder::new(0..100)
            .title("Q: What number do you like?")
            .title_color(style::Color::DarkGreen)
            .query()
            .build()?;
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
